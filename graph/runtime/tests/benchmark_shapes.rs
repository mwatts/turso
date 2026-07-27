use std::collections::HashSet;

use serde::Deserialize;
use turso_graph_ir::{NodeId, RelationshipId, RelationshipTypeId, RoleId};
use turso_graph_runtime::{BuildLimits, EdgeInput, Graph, LimitKind, RuntimeError};

const MANIFEST: &str = include_str!("../../testdata/benchmarks/manifest.toml");

#[derive(Deserialize)]
struct Manifest {
    version: u32,
    purpose: String,
    shape: Vec<ShapeFixture>,
}

#[derive(Deserialize)]
struct ShapeFixture {
    name: String,
    nodes: u64,
    relationships: u64,
    definition: String,
}

fn fixture(shape: &str, node_count: u64) -> (Vec<NodeId>, Vec<EdgeInput>) {
    let nodes = (1..=node_count).map(node).collect::<Vec<_>>();
    let pairs: Vec<(u64, u64)> = match shape {
        "sparse" => (2..=node_count)
            .map(|target| (target - 1, target))
            .collect(),
        "dense" => (1..=node_count)
            .flat_map(|source| {
                (1..=node_count)
                    .filter(move |target| *target != source)
                    .map(move |target| (source, target))
            })
            .collect(),
        "skewed" => (2..=node_count)
            .flat_map(|target| [(1, target), (target - 1, target)])
            .collect(),
        "cyclic" => (1..=node_count)
            .map(|source| (source, source % node_count + 1))
            .collect(),
        "high-degree" => (2..=node_count).map(|target| (1, target)).collect(),
        shape => panic!("unknown benchmark shape {shape}"),
    };
    let edges = pairs
        .into_iter()
        .enumerate()
        .map(|(index, (source, target))| EdgeInput {
            relationship: RelationshipId::new(index as u64 + 1).unwrap(),
            from_role: RoleId::new(1).unwrap(),
            to_role: RoleId::new(2).unwrap(),
            source: node(source),
            target: node(target),
            relationship_type: RelationshipTypeId::new(1).unwrap(),
            weight: None,
        })
        .collect();
    (nodes, edges)
}

fn node(value: u64) -> NodeId {
    NodeId::new(value).unwrap()
}

#[test]
fn all_representative_shapes_are_discovered_and_reproducible() {
    let manifest: Manifest = toml::from_str(MANIFEST).unwrap();
    assert_eq!(manifest.version, 1);
    assert!(!manifest.purpose.is_empty());
    assert!(
        !manifest.shape.is_empty(),
        "zero benchmark discovery must fail"
    );
    let mut observed = HashSet::new();
    for shape in manifest.shape {
        assert!(!shape.definition.is_empty());
        assert!(observed.insert(shape.name.clone()));
        let (nodes, edges) = fixture(&shape.name, shape.nodes);
        assert_eq!(nodes.len() as u64, shape.nodes, "{} nodes", shape.name);
        assert_eq!(
            edges.len() as u64,
            shape.relationships,
            "{} relationships",
            shape.name
        );
        let graph = Graph::build(
            nodes.iter().copied(),
            edges.iter().copied(),
            BuildLimits::default(),
        )
        .unwrap();
        assert_eq!(graph.node_count() as u64, shape.nodes);
        assert_eq!(graph.edge_count() as u64, shape.relationships);
    }
    assert_eq!(
        observed,
        ["sparse", "dense", "skewed", "cyclic", "high-degree"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
}

#[test]
fn every_shape_is_bounded_by_node_edge_and_memory_caps() {
    let manifest: Manifest = toml::from_str(MANIFEST).unwrap();
    for shape in manifest.shape {
        let (nodes, edges) = fixture(&shape.name, shape.nodes);
        assert!(matches!(
            Graph::build(
                nodes.iter().copied(),
                edges.iter().copied(),
                BuildLimits {
                    max_nodes: shape.nodes - 1,
                    ..BuildLimits::default()
                }
            ),
            Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Nodes,
                ..
            })
        ));
        assert!(matches!(
            Graph::build(
                nodes.iter().copied(),
                edges.iter().copied(),
                BuildLimits {
                    max_edges: shape.relationships - 1,
                    ..BuildLimits::default()
                }
            ),
            Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Edges,
                ..
            })
        ));
        let graph = Graph::build(
            nodes.iter().copied(),
            edges.iter().copied(),
            BuildLimits::default(),
        )
        .unwrap();
        assert!(matches!(
            Graph::build(
                nodes.iter().copied(),
                edges.iter().copied(),
                BuildLimits {
                    max_memory_bytes: graph.estimated_heap_bytes() - 1,
                    ..BuildLimits::default()
                }
            ),
            Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Memory,
                ..
            })
        ));
    }
}
