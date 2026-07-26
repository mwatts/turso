use divan::{black_box, Bencher};
use turso_graph_ir::{NodeId, RelationshipId, RelationshipTypeId, RoleId};
use turso_graph_runtime::{BuildLimits, EdgeInput, Graph};

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum Shape {
    Sparse,
    Dense,
    Skewed,
    Cyclic,
    HighDegree,
}

fn fixture(shape: Shape, node_count: u64) -> (Vec<NodeId>, Vec<EdgeInput>) {
    let nodes = (1..=node_count).map(node).collect::<Vec<_>>();
    let pairs: Vec<(u64, u64)> = match shape {
        Shape::Sparse => (2..=node_count)
            .map(|target| (target - 1, target))
            .collect(),
        Shape::Dense => (1..=node_count)
            .flat_map(|source| {
                (1..=node_count)
                    .filter(move |target| *target != source)
                    .map(move |target| (source, target))
            })
            .collect(),
        Shape::Skewed => (2..=node_count)
            .flat_map(|target| [(1, target), (target - 1, target)])
            .collect(),
        Shape::Cyclic => (1..=node_count)
            .map(|source| (source, source % node_count + 1))
            .collect(),
        Shape::HighDegree => (2..=node_count).map(|target| (1, target)).collect(),
    };
    let edges = pairs
        .into_iter()
        .enumerate()
        .map(|(index, (source, target))| EdgeInput {
            relationship: RelationshipId::new(index as u64 + 1).unwrap(),
            from_role: role(1),
            to_role: role(2),
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

fn role(value: u32) -> RoleId {
    RoleId::new(value).unwrap()
}

fn bench_build(bencher: Bencher, shape: Shape, node_count: u64) {
    let (nodes, edges) = fixture(shape, node_count);
    bencher.bench_local(|| {
        black_box(
            Graph::build(
                nodes.iter().copied(),
                edges.iter().copied(),
                BuildLimits::default(),
            )
            .unwrap(),
        )
    });
}

#[turso_macros::divan_bench]
fn sparse_10k(bencher: Bencher) {
    bench_build(bencher, Shape::Sparse, 10_000);
}

#[turso_macros::divan_bench]
fn dense_250(bencher: Bencher) {
    bench_build(bencher, Shape::Dense, 250);
}

#[turso_macros::divan_bench]
fn skewed_10k(bencher: Bencher) {
    bench_build(bencher, Shape::Skewed, 10_000);
}

#[turso_macros::divan_bench]
fn cyclic_10k(bencher: Bencher) {
    bench_build(bencher, Shape::Cyclic, 10_000);
}

#[turso_macros::divan_bench]
fn high_degree_10k(bencher: Bencher) {
    bench_build(bencher, Shape::HighDegree, 10_000);
}
