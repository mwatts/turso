mod fixture;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fixture::{bind_fixture, first_role_expand};
use serde::Deserialize;
use turso_core::{Database, MemoryIO, SqliteDialect, Value};
use turso_graph_cypher::parse;
use turso_graph_frontend::{
    bind, graph_frontend_id, lower_relational, CatalogEntity, GraphCatalogSnapshot, GraphCompiler,
    NodeTableLayout, ParameterTypes, RelationalCatalogSnapshot, RelationshipRoleLayout,
    RelationshipTableLayout, ResolvedProperty,
};
use turso_graph_ir::{
    GraphId, LabelId, Nullability, PropertyId, RelationshipTypeId, RoleCardinality, RoleId,
    SourceTableId, ValueType,
};

const MANIFEST: &str = include_str!("../../testdata/fixed-patterns/manifest.toml");

#[derive(Deserialize)]
struct Manifest {
    version: u32,
    purpose: String,
    fixture: Vec<Fixture>,
}

#[derive(Deserialize)]
struct Fixture {
    id: String,
    query: String,
    ordering: String,
    parser_status: String,
    unsupported: Option<String>,
    source_repo: String,
    revision: String,
    source_path: String,
    source_case: String,
    license: String,
    adaptation: String,
}

struct Catalog;

impl GraphCatalogSnapshot for Catalog {
    fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(1).ok()
    }

    fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(2).ok()
    }

    fn label(&self, _graph: GraphId, _name: &str) -> Option<LabelId> {
        LabelId::new(1).ok()
    }

    fn relationship_type(&self, _graph: GraphId, _name: &str) -> Option<RelationshipTypeId> {
        RelationshipTypeId::new(1).ok()
    }

    fn property(
        &self,
        _graph: GraphId,
        _entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        let id = match name {
            "name" => 1,
            "age" => 2,
            "val" => 3,
            _ => 4,
        };
        Some(ResolvedProperty {
            id: PropertyId::new(id).ok()?,
            value_type: if name == "age" || name == "val" {
                ValueType::Integer
            } else {
                ValueType::Text
            },
            nullability: Nullability::Nullable,
        })
    }

    fn relationship_source_roles(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
        self.relationship_layout(source)
    }
}

impl RelationalCatalogSnapshot for Catalog {
    fn node_layout(&self, source: SourceTableId) -> Option<NodeTableLayout> {
        (source.get() == 1).then(|| NodeTableLayout {
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        })
    }

    fn relationship_layout(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
        (source.get() == 2).then(|| RelationshipTableLayout {
            table: "relationships".to_owned(),
            identity_column: "id".to_owned(),
            roles: vec![
                RelationshipRoleLayout {
                    role: RoleId::new(1).unwrap(),
                    name: "start".to_owned(),
                    column: "src".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
                RelationshipRoleLayout {
                    role: RoleId::new(2).unwrap(),
                    name: "end".to_owned(),
                    column: "dst".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
            ],
        })
    }

    fn property_column(&self, _source: SourceTableId, property: PropertyId) -> Option<String> {
        match property.get() {
            1 => Some("name".to_owned()),
            2 => Some("age".to_owned()),
            3 => Some("val".to_owned()),
            _ => None,
        }
    }
}

fn manifest() -> Manifest {
    toml::from_str(MANIFEST).expect("fixed-pattern manifest must be valid TOML")
}

fn role(value: u32) -> RoleId {
    RoleId::new(value).unwrap()
}

#[test]
fn fixture_manifest_has_provenance_ordering_and_explicit_support_status() {
    let manifest = manifest();
    assert_eq!(manifest.version, 1);
    assert!(!manifest.purpose.is_empty());
    assert!(manifest.fixture.len() >= 11);
    let mut ids = HashSet::new();
    for fixture in manifest.fixture {
        assert!(ids.insert(fixture.id.clone()));
        assert!(matches!(fixture.ordering.as_str(), "ordered" | "unordered"));
        // SEMANTIC_PROFILE.row_order is OrderedOnlyUnderExplicitOrderBy. A
        // fixture that claims an order its query never asked for documents a
        // guarantee Turso does not make: UNWIND, for one, lowers to a bare
        // `JOIN json_each(...)` with no ORDER BY, so its row order is a query
        // plan artifact that a planner change may reorder.
        assert!(
            fixture.ordering != "ordered"
                || fixture.query.to_ascii_uppercase().contains("ORDER BY"),
            "fixture `{}` claims ordered without ORDER BY: {}",
            fixture.id,
            fixture.query
        );
        assert!(matches!(
            fixture.parser_status.as_str(),
            "supported" | "unsupported"
        ));
        assert_eq!(
            fixture.unsupported.is_some(),
            fixture.parser_status == "unsupported"
        );
        assert!(fixture.source_repo.starts_with("https://github.com/"));
        assert_eq!(fixture.revision.len(), 40);
        assert!(!fixture.source_path.is_empty());
        assert!(!fixture.source_case.is_empty());
        assert_eq!(fixture.license, "Apache-2.0");
        assert_eq!(fixture.adaptation, "fixture-adaptation");
    }
}

#[test]
fn required_fixtures_reach_relational_lowering() {
    let parameters = HashMap::from([("name".to_owned(), (ValueType::Text, Nullability::NonNull))]);
    for fixture in manifest().fixture {
        let parsed = parse(&fixture.query);
        if fixture.parser_status == "unsupported" {
            assert!(
                parsed.is_err(),
                "{} is tagged unsupported but parsed",
                fixture.id
            );
            continue;
        }
        let query = parsed.unwrap_or_else(|error| panic!("{} did not parse: {error}", fixture.id));
        let bound = bind(
            &query,
            GraphId::new(1).expect("graph id"),
            &Catalog,
            &parameters as &ParameterTypes,
        )
        .unwrap_or_else(|error| panic!("{} did not bind: {error}", fixture.id));
        lower_relational(&bound.plan, &Catalog)
            .unwrap_or_else(|error| panic!("{} did not lower: {error}", fixture.id));
    }
}

#[test]
fn supported_fixtures_execute_through_turso_planner_and_vdbe() {
    let io = Arc::new(MemoryIO::new());
    let connection = Database::open_file(io, ":memory:graph-lowering", Arc::new(SqliteDialect))
        .expect("open graph fixture database")
        .connect()
        .expect("connect graph fixture database");
    connection
        .execute(
            "CREATE TABLE people(\
                 id INTEGER PRIMARY KEY, name TEXT, age INTEGER, val INTEGER\
             );\
             CREATE TABLE relationships(\
                 id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER\
             );\
             INSERT INTO people VALUES\
                 (1, 'Alix', 30, 1),\
                 (2, 'Bea', 20, 2),\
                 (3, 'Cy', 40, 3),\
                 (4, 'Dee', 25, 4);\
             INSERT INTO relationships VALUES (1, 1, 2), (2, 2, 3);",
        )
        .expect("create graph fixture data");

    let parameters = HashMap::from([("name".to_owned(), (ValueType::Text, Nullability::NonNull))]);
    connection
        .register_frontend_compiler(
            graph_frontend_id(),
            Arc::new(GraphCompiler::new(
                GraphId::new(1).expect("graph id"),
                Arc::new(Catalog),
                parameters,
            )),
        )
        .expect("register graph compiler");

    let mut observed = HashMap::new();
    for fixture in manifest().fixture {
        assert_eq!(fixture.parser_status, "supported");
        let mut statement = connection
            .prepare_frontend(&graph_frontend_id(), &fixture.query)
            .unwrap_or_else(|error| panic!("{} did not prepare: {error}", fixture.id));
        if let Some(index) = statement.parameter_index("$name") {
            statement
                .bind_at(index, Value::build_text("Alix"))
                .expect("bind fixture parameter");
        }
        let rows = statement
            .run_collect_rows()
            .unwrap_or_else(|error| panic!("{} did not execute: {error}", fixture.id));
        observed.insert(
            fixture.id,
            rows.into_iter()
                .map(|row| row.into_iter().map(|value| value.to_string()).collect())
                .collect::<Vec<Vec<String>>>(),
        );
    }

    assert_eq!(
        observed["unwind-list"],
        vec![vec!["1"], vec!["2"], vec!["3"]]
    );
    assert_eq!(
        observed["fixed-directed-edge"],
        vec![vec!["Alix", "Bea"], vec!["Bea", "Cy"]]
    );
    assert_eq!(observed["fixed-undirected-edge"], vec![vec!["Bea"]]);
    assert_eq!(observed["fixed-multi-hop"], vec![vec!["1", "2", "3"]]);
    assert_eq!(observed["property-predicate-parameter"], vec![vec!["Alix"]]);
    assert_eq!(
        observed["order-by-projected-row"],
        vec![vec!["Bea"], vec!["Dee"], vec!["Alix"], vec!["Cy"]]
    );
    assert_eq!(observed["skip-and-limit"], vec![vec!["2"], vec!["3"]]);
    assert_eq!(observed["aggregate-count"], vec![vec!["4"]]);

    let incoming = "MATCH (a:Person)<-[:KNOWS]-(b:Person) \
                    RETURN a.name, b.name ORDER BY a.name";
    let rows = connection
        .prepare_frontend(&graph_frontend_id(), incoming)
        .expect("prepare incoming expansion")
        .run_collect_rows()
        .expect("execute incoming expansion");
    assert_eq!(
        rows,
        vec![
            vec![Value::build_text("Bea"), Value::build_text("Alix")],
            vec![Value::build_text("Cy"), Value::build_text("Bea")],
        ],
        "incoming expansion must reverse the relationship endpoints"
    );

    let optional_with_predicate = "MATCH (a:Person {name: 'Alix'}) \
         OPTIONAL MATCH (a)-[r:KNOWS]->(b) WHERE b.age > 100 \
         RETURN a.name, r, b.name";
    let rows = connection
        .prepare_frontend(&graph_frontend_id(), optional_with_predicate)
        .expect("prepare optional predicate")
        .run_collect_rows()
        .expect("execute optional predicate");
    assert_eq!(
        rows,
        vec![vec![Value::build_text("Alix"), Value::Null, Value::Null]],
        "an optional predicate must preserve the input row and null the entire unmatched pattern"
    );
}

#[test]
fn an_outgoing_expand_binds_the_start_to_end_role_pair() {
    // The role pair must agree with the direction it is replacing, or the
    // contract half of this migration silently reverses every traversal.
    let plan = bind_fixture("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b");
    let expand = first_role_expand(&plan);
    assert_eq!(expand.from_role.get(), 1, "role 1 is `start`");
    assert_eq!(expand.to_role.get(), 2, "role 2 is `end`");
    assert!(!expand.symmetric);
}

#[test]
fn an_incoming_expand_reverses_the_role_pair_rather_than_flagging_it() {
    let plan = bind_fixture("MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b");
    let expand = first_role_expand(&plan);
    assert_eq!(expand.role_pair(), (role(2), role(1)));
    assert!(!expand.symmetric);
}

#[test]
fn an_undirected_same_source_expand_is_the_symmetric_pair() {
    // Today's Direction::Both. The binder only emits it when both endpoints
    // come from one node source; otherwise it unions two directed branches,
    // and this test would find two expands rather than a symmetric one.
    let plan = bind_fixture("MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b");
    let expand = first_role_expand(&plan);
    assert_eq!(expand.role_pair(), (role(1), role(2)));
    assert!(
        expand.symmetric,
        "an undirected pattern matches the pair in both orders"
    );
}
