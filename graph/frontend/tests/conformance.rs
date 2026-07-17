use std::collections::HashSet;
use std::sync::Arc;

use serde::Deserialize;
use turso_core::{Database, MemoryIO, SqliteDialect, Value};
use turso_graph_frontend::{
    register_graph, CatalogEntity, GraphCatalogSnapshot, GraphCompilationCatalog,
    GraphRegistration, GraphSession, MutationParameters, NodeSourceRegistration, NodeTableLayout,
    ParameterTypes, RelationalCatalogSnapshot, RelationshipSourceRegistration,
    RelationshipTableLayout, ResolvedProperty, SnapshotStore,
};
use turso_graph_ir as ir;

const MANIFEST: &str = include_str!("../../testdata/conformance/manifest.toml");
const REPORT: &str = include_str!("../../CONFORMANCE.md");

#[derive(Deserialize)]
struct Manifest {
    version: u32,
    purpose: String,
    scenario: Vec<Scenario>,
}

#[derive(Deserialize)]
struct Scenario {
    id: String,
    feature: String,
    status: String,
    action: String,
    ordering: String,
    query: String,
    verification_query: Option<String>,
    #[serde(default)]
    expected_rows: Vec<Vec<String>>,
    unsupported_reason: Option<String>,
    source: String,
    source_repo: String,
    revision: String,
    source_path: String,
    source_case: String,
    license: String,
}

struct Catalog {
    node_source: ir::SourceTableId,
    relationship_source: ir::SourceTableId,
}

impl GraphCatalogSnapshot for Catalog {
    fn node_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
        Some(self.node_source)
    }

    fn relationship_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
        Some(self.relationship_source)
    }

    fn label(&self, _graph: ir::GraphId, name: &str) -> Option<ir::LabelId> {
        (name == "Person").then(|| ir::LabelId::new(1).unwrap())
    }

    fn relationship_type(&self, _graph: ir::GraphId, name: &str) -> Option<ir::RelationshipTypeId> {
        (name == "KNOWS").then(|| ir::RelationshipTypeId::new(1).unwrap())
    }

    fn property(
        &self,
        _graph: ir::GraphId,
        entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        let (id, value_type, nullability) = match (entity, name) {
            (CatalogEntity::Node, "id") => (1, ir::ValueType::Integer, ir::Nullability::NonNull),
            (CatalogEntity::Node, "name") => (2, ir::ValueType::Text, ir::Nullability::Nullable),
            (CatalogEntity::Node, "age") => (3, ir::ValueType::Integer, ir::Nullability::Nullable),
            _ => return None,
        };
        Some(ResolvedProperty {
            id: ir::PropertyId::new(id).unwrap(),
            value_type,
            nullability,
        })
    }
}

impl RelationalCatalogSnapshot for Catalog {
    fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout> {
        (source == self.node_source).then(|| NodeTableLayout {
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        })
    }

    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout> {
        (source == self.relationship_source).then(|| RelationshipTableLayout {
            table: "relationships".to_owned(),
            identity_column: "id".to_owned(),
            start_column: "src".to_owned(),
            end_column: "dst".to_owned(),
        })
    }

    fn property_column(
        &self,
        source: ir::SourceTableId,
        property: ir::PropertyId,
    ) -> Option<String> {
        match (source, property.get()) {
            (source, 1) if source == self.node_source => Some("id".to_owned()),
            (source, 2) if source == self.node_source => Some("name".to_owned()),
            (source, 3) if source == self.node_source => Some("age".to_owned()),
            _ => None,
        }
    }
}

fn manifest() -> Manifest {
    toml::from_str(MANIFEST).expect("conformance manifest must be valid TOML")
}

fn session(id: &str) -> GraphSession {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        &format!(":memory:graph-conformance-{id}"),
        Arc::new(SqliteDialect),
    )
    .unwrap();
    let connection = database.connect().unwrap();
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER); \
             INSERT INTO people VALUES \
                 (1, 'Ada', 40), (2, 'Grace', 35), (3, 'Linus', 30), (4, 'Edsger', 50); \
             INSERT INTO relationships VALUES (10, 1, 2), (20, 2, 3), (30, 1, 3)",
        )
        .unwrap();
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "KNOWS".to_owned(),
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
                start_node_source: "Person".to_owned(),
                end_node_source: "Person".to_owned(),
            }],
        },
    )
    .unwrap();
    let catalog: Arc<dyn GraphCompilationCatalog> = Arc::new(Catalog {
        node_source: registered.node_sources[0].id,
        relationship_source: registered.relationship_sources[0].id,
    });
    GraphSession::install(
        connection,
        &registered,
        catalog,
        ParameterTypes::new(),
        Arc::new(SnapshotStore::default()),
        Default::default(),
    )
    .unwrap()
}

fn normalized_rows(rows: Vec<Vec<Value>>, ordering: &str) -> Vec<Vec<String>> {
    let mut rows = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|value| match value {
                    Value::Null => "<null>".to_owned(),
                    value => value.to_string(),
                })
                .collect()
        })
        .collect::<Vec<_>>();
    if ordering == "unordered" {
        rows.sort();
    }
    rows
}

fn render_report(
    supported: &[String],
    failed: &[String],
    unsupported: &[(String, String)],
) -> String {
    let mut report = String::from(
        "# Turso graph conformance report\n\n\
         Generated from `graph/testdata/conformance/manifest.toml`. Supported scenarios execute \
         end-to-end; unordered results compare as multisets. Unsupported scenarios must fail at \
         the frontend boundary. A supported scenario that errors or returns different rows is \
         reported separately as failed and fails CI. This curated mixed-source slice is not a \
         claim of full openCypher TCK conformance.\n\n",
    );
    report.push_str(&format!("## Supported ({})\n\n", supported.len()));
    for id in supported {
        report.push_str(&format!("- `{id}`\n"));
    }
    report.push_str(&format!("\n## Failed ({})\n\n", failed.len()));
    if failed.is_empty() {
        report.push_str("- None.\n");
    } else {
        for failure in failed {
            report.push_str(&format!("- {failure}\n"));
        }
    }
    report.push_str(&format!("\n## Unsupported ({})\n\n", unsupported.len()));
    for (id, reason) in unsupported {
        report.push_str(&format!("- `{id}` — {reason}.\n"));
    }
    report
}

#[test]
fn manifest_has_complete_mixed_source_provenance_and_nonzero_discovery() {
    let manifest = manifest();
    assert_eq!(manifest.version, 1);
    assert!(!manifest.purpose.is_empty());
    assert!(!manifest.scenario.is_empty(), "zero discovery must fail");
    let required_sources = [
        "openCypher TCK via Uni",
        "Grafeo",
        "Apache AGE",
        "pgGraph",
        "Ladybug",
        "SparrowDB",
        "CQLite",
        "Samyama",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<HashSet<_>>();
    let mut observed_sources = HashSet::new();
    let mut ids = HashSet::new();
    let mut supported = 0;
    let mut unsupported = 0;
    for scenario in manifest.scenario {
        assert!(ids.insert(scenario.id));
        assert!(!scenario.feature.is_empty());
        assert!(matches!(
            scenario.status.as_str(),
            "supported" | "unsupported"
        ));
        assert!(matches!(scenario.action.as_str(), "query" | "mutation"));
        assert!(matches!(
            scenario.ordering.as_str(),
            "ordered" | "unordered"
        ));
        assert_eq!(
            scenario.unsupported_reason.is_some(),
            scenario.status == "unsupported"
        );
        assert!(scenario.source_repo.starts_with("https://github.com/"));
        assert_eq!(scenario.revision.len(), 40);
        assert!(!scenario.source_path.is_empty());
        assert!(!scenario.source_case.is_empty());
        assert!(matches!(scenario.license.as_str(), "Apache-2.0" | "MIT"));
        observed_sources.insert(scenario.source);
        if scenario.status == "supported" {
            supported += 1;
        } else {
            unsupported += 1;
        }
    }
    assert_eq!(observed_sources, required_sources);
    assert!(supported > 0);
    assert!(unsupported > 0);
}

#[test]
fn supported_failed_and_unsupported_scenarios_are_reported_separately() {
    let mut supported = Vec::new();
    let mut failed = Vec::new();
    let mut unsupported = Vec::new();

    for scenario in manifest().scenario {
        let session = session(&scenario.id);
        if scenario.status == "unsupported" {
            let result = session.query(&scenario.query, &MutationParameters::new());
            if result.is_err() {
                unsupported.push((
                    scenario.id,
                    scenario
                        .unsupported_reason
                        .expect("unsupported scenario needs a reason"),
                ));
            } else {
                failed.push(format!("{} unexpectedly succeeded", scenario.id));
            }
            continue;
        }

        let result = match scenario.action.as_str() {
            "query" => session.query(&scenario.query, &MutationParameters::new()),
            "mutation" => session
                .mutate(&scenario.query, &MutationParameters::new())
                .and_then(|_| {
                    session.query(
                        scenario
                            .verification_query
                            .as_deref()
                            .expect("mutation scenario needs verification_query"),
                        &MutationParameters::new(),
                    )
                }),
            _ => unreachable!(),
        };
        match result {
            Ok(rows) => {
                let observed = normalized_rows(rows, &scenario.ordering);
                let mut expected = scenario.expected_rows;
                if scenario.ordering == "unordered" {
                    expected.sort();
                }
                if observed == expected {
                    supported.push(scenario.id);
                } else {
                    failed.push(format!(
                        "{} rows: expected {expected:?}, observed {observed:?}",
                        scenario.id
                    ));
                }
            }
            Err(error) => failed.push(format!("{}: {error}", scenario.id)),
        }
    }

    assert_eq!(supported.len(), 12, "supported: {supported:?}");
    assert_eq!(unsupported.len(), 6, "unsupported: {unsupported:?}");
    assert!(failed.is_empty(), "failed scenarios: {failed:#?}");
    assert_eq!(render_report(&supported, &failed, &unsupported), REPORT);
}
