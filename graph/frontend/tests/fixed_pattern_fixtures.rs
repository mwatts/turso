use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use turso_graph_cypher::parse;
use turso_graph_frontend::{
    bind, lower_relational, CatalogEntity, GraphCatalogSnapshot, ParameterTypes, RegisteredGraph,
    RegisteredNodeSource, RegisteredRelationshipSource, ResolvedProperty,
};
use turso_graph_ir::{
    GraphId, LabelId, Nullability, PropertyId, RelationshipTypeId, SourceTableId, ValueType,
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
}

fn manifest() -> Manifest {
    toml::from_str(MANIFEST).expect("fixed-pattern manifest must be valid TOML")
}

fn registered_graph() -> RegisteredGraph {
    RegisteredGraph {
        id: GraphId::new(1).expect("graph id"),
        name: "fixture".to_owned(),
        generation: 0,
        node_sources: vec![RegisteredNodeSource {
            id: SourceTableId::new(1).expect("node source"),
            name: "Person".to_owned(),
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        }],
        relationship_sources: vec![RegisteredRelationshipSource {
            id: SourceTableId::new(2).expect("relationship source"),
            name: "KNOWS".to_owned(),
            table: "relationships".to_owned(),
            start_column: "src".to_owned(),
            end_column: "dst".to_owned(),
            start_node_source: SourceTableId::new(1).expect("start source"),
            end_node_source: SourceTableId::new(1).expect("end source"),
        }],
    }
}

#[test]
fn fixture_manifest_has_provenance_ordering_and_explicit_support_status() {
    let manifest = manifest();
    assert_eq!(manifest.version, 1);
    assert!(!manifest.purpose.is_empty());
    assert!(manifest.fixture.len() >= 11);
    let mut ids = HashSet::new();
    for fixture in manifest.fixture {
        assert!(ids.insert(fixture.id));
        assert!(matches!(fixture.ordering.as_str(), "ordered" | "unordered"));
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
    let graph = registered_graph();
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
        lower_relational(&bound.plan, &graph)
            .unwrap_or_else(|error| panic!("{} did not lower: {error}", fixture.id));
    }
}
