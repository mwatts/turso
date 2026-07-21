use serde::Deserialize;
use turso_graph_ir::{Direction, NodeId, RelationshipId, RelationshipTypeId};
use turso_graph_runtime::{
    shortest_path, traverse, weighted_shortest_path, BuildLimits, EdgeInput, Graph, NeverCancelled,
    ShortestPathRequest, TraversalLimits, TraversalOrder, TraversalRequest, Uniqueness,
};

const MANIFEST: &str = include_str!("../../testdata/pggraph-runtime/manifest.toml");
const REVISION: &str = "d689bcf2b3b52d7f878f61718be69ebcb953affc";

#[derive(Deserialize)]
struct Manifest {
    version: u32,
    purpose: String,
    fixture: Vec<Fixture>,
}

#[derive(Deserialize)]
struct Fixture {
    id: String,
    algorithm: String,
    nodes: Vec<u64>,
    edges: Vec<EdgeFixture>,
    start: u64,
    target: Option<u64>,
    max_hops: u32,
    expected_paths: Vec<Vec<u64>>,
    expected_weight: Option<u64>,
    source_path: String,
    source_case: String,
    revision: String,
    license: String,
}

#[derive(Deserialize)]
struct EdgeFixture {
    id: u64,
    source: u64,
    target: u64,
    kind: u32,
    weight: u64,
}

fn node(value: u64) -> NodeId {
    NodeId::new(value).expect("fixture node id must be non-zero")
}

fn graph(fixture: &Fixture) -> Graph {
    Graph::build(
        fixture.nodes.iter().copied().map(node),
        fixture.edges.iter().map(|edge| EdgeInput {
            relationship: RelationshipId::new(edge.id).expect("relationship id"),
            source: node(edge.source),
            target: node(edge.target),
            relationship_type: RelationshipTypeId::new(edge.kind).expect("relationship type"),
            weight: Some(edge.weight),
        }),
        BuildLimits::default(),
    )
    .expect("fixture graph must build")
}

#[test]
fn normalized_cases_match_the_pinned_pggraph_behavior() {
    let manifest: Manifest = toml::from_str(MANIFEST).expect("runtime manifest must parse");
    assert_eq!(manifest.version, 1);
    assert!(!manifest.purpose.is_empty());
    assert!(!manifest.fixture.is_empty());

    for fixture in manifest.fixture {
        assert_eq!(fixture.revision, REVISION, "{} revision", fixture.id);
        assert_eq!(fixture.license, "Apache-2.0", "{} license", fixture.id);
        assert!(fixture.source_path.starts_with("graph/src/"));
        assert!(!fixture.source_case.is_empty());
        let graph = graph(&fixture);
        let (mut observed, weight) = match fixture.algorithm.as_str() {
            "incoming" => {
                let paths: Vec<Vec<u64>> = graph
                    .neighbors(node(fixture.start), Direction::Incoming, &[])
                    .expect("incoming neighbors")
                    .into_iter()
                    .map(|neighbor| vec![fixture.start, neighbor.node.get()])
                    .collect();
                (paths, None)
            }
            "traverse" => {
                let paths = traverse(
                    &graph,
                    &TraversalRequest {
                        start: node(fixture.start),
                        direction: Direction::Outgoing,
                        relationship_types: vec![],
                        min_hops: 1,
                        max_hops: fixture.max_hops,
                        error_at_max_hops: false,
                        uniqueness: Uniqueness::Trail,
                        order: TraversalOrder::BreadthFirst,
                    },
                    TraversalLimits::default(),
                    &NeverCancelled,
                )
                .expect("bounded traversal")
                .into_iter()
                .map(|path| path.nodes.into_iter().map(NodeId::get).collect())
                .collect();
                (paths, None)
            }
            "shortest" | "weighted" => {
                let request = ShortestPathRequest {
                    source: node(fixture.start),
                    target: node(fixture.target.expect("shortest target")),
                    direction: Direction::Outgoing,
                    relationship_types: vec![],
                    max_hops: fixture.max_hops,
                };
                let path = if fixture.algorithm == "shortest" {
                    shortest_path(
                        &graph,
                        &request,
                        TraversalLimits::default(),
                        &NeverCancelled,
                    )
                } else {
                    weighted_shortest_path(
                        &graph,
                        &request,
                        TraversalLimits::default(),
                        &NeverCancelled,
                    )
                }
                .expect("shortest path");
                let weight = if fixture.algorithm == "weighted" {
                    path.as_ref().map(|path| path.total_weight)
                } else {
                    None
                };
                let paths = path
                    .into_iter()
                    .map(|path| path.nodes.into_iter().map(NodeId::get).collect())
                    .collect();
                (paths, weight)
            }
            algorithm => panic!("unknown fixture algorithm {algorithm}"),
        };
        observed.sort();
        assert_eq!(observed, fixture.expected_paths, "{} paths", fixture.id);
        assert_eq!(weight, fixture.expected_weight, "{} weight", fixture.id);
    }
}
