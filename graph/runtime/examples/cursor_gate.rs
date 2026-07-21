use std::time::{Duration, Instant};

use turso_graph_ir::{Direction, NodeId, RelationshipId, RelationshipTypeId};
use turso_graph_runtime::{
    BuildLimits, EdgeInput, Graph, NeverCancelled, TraversalCursor, TraversalLimits,
    TraversalOrder, TraversalRequest, TraversalStep, Uniqueness,
};

const FANOUT: u64 = 100_000;
const WORK_QUANTUM: u64 = 256;

fn main() {
    let nodes = (1..=FANOUT + 1).map(node).collect::<Vec<_>>();
    let edges = (2..=FANOUT + 1)
        .map(|target| EdgeInput {
            relationship: relationship(target - 1),
            source: node(1),
            target: node(target),
            relationship_type: relationship_type(1),
            weight: None,
        })
        .collect::<Vec<_>>();
    let graph = Graph::build(nodes, edges, BuildLimits::default()).expect("build benchmark graph");
    let request = TraversalRequest {
        start: node(1),
        direction: Direction::Outgoing,
        relationship_types: Vec::new(),
        min_hops: 2,
        max_hops: 2,
        error_at_max_hops: false,
        uniqueness: Uniqueness::Trail,
        order: TraversalOrder::BreadthFirst,
    };
    let mut cursor = TraversalCursor::new(&graph, request, TraversalLimits::default())
        .expect("create traversal cursor");

    let started = Instant::now();
    let mut calls = 0_u64;
    let mut pending = 0_u64;
    let mut max_call = Duration::ZERO;
    loop {
        let call_started = Instant::now();
        let step = cursor
            .step(&graph, &NeverCancelled, WORK_QUANTUM)
            .expect("advance traversal cursor");
        max_call = max_call.max(call_started.elapsed());
        calls += 1;
        match step {
            TraversalStep::Pending => pending += 1,
            TraversalStep::Done => break,
            TraversalStep::Path(path) => panic!("star graph unexpectedly produced {path:?}"),
        }
    }
    println!(
        "fanout={FANOUT} quantum={WORK_QUANTUM} calls={calls} pending={pending} total_us={} max_call_us={}",
        started.elapsed().as_micros(),
        max_call.as_micros()
    );
}

fn node(value: u64) -> NodeId {
    NodeId::new(value).expect("non-zero node")
}

fn relationship(value: u64) -> RelationshipId {
    RelationshipId::new(value).expect("non-zero relationship")
}

fn relationship_type(value: u32) -> RelationshipTypeId {
    RelationshipTypeId::new(value).expect("non-zero relationship type")
}
