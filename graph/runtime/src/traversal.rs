// Source: https://github.com/Evokoa/pgGraph
// Revision: d689bcf2b3b52d7f878f61718be69ebcb953affc
// Path: graph/src/bfs.rs
// License: Apache-2.0
// Adaptation: structural-adaptation
// Changes: Replaced pgGraph node stores, roaring bitmaps, overlays, tenants,
// and GUC circuit breakers with typed paths over Turso identities, explicit
// per-call budgets, cancellation, and walk/trail/node-simple uniqueness.

use std::collections::VecDeque;
use std::mem::size_of;

use turso_graph_ir::{Direction, NodeId, RelationshipId, RelationshipTypeId};

use crate::{limits::Budget, Cancellation, Graph, RuntimeError, RuntimeResult, TraversalLimits};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraversalOrder {
    BreadthFirst,
    DepthFirst,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Uniqueness {
    Walk,
    Trail,
    Path,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraversalRequest {
    pub start: NodeId,
    pub direction: Direction,
    pub relationship_types: Vec<RelationshipTypeId>,
    pub min_hops: u32,
    pub max_hops: u32,
    pub uniqueness: Uniqueness,
    pub order: TraversalOrder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Path {
    pub nodes: Vec<NodeId>,
    pub relationships: Vec<RelationshipId>,
    pub relationship_types: Vec<RelationshipTypeId>,
    pub total_weight: u64,
}

/// Resumable traversal state used by execution adapters that cannot
/// materialize every matching path before returning the first row.
pub struct TraversalCursor {
    request: TraversalRequest,
    budget: Budget,
    frontier: Frontier,
    finished: bool,
}

impl TraversalCursor {
    pub fn new(
        graph: &Graph,
        request: TraversalRequest,
        limits: TraversalLimits,
    ) -> RuntimeResult<Self> {
        validate_request(graph, &request)?;
        let mut budget = Budget::new(limits)?;
        budget.require_hops(request.max_hops)?;
        budget.node()?;

        let initial = PathState::start(request.start);
        budget.retain_memory(initial.memory_bytes())?;
        let mut frontier = Frontier::new(request.order);
        frontier.push(initial);
        Ok(Self {
            request,
            budget,
            frontier,
            finished: false,
        })
    }

    /// Advance until the next matching path or exhaustion while preserving
    /// the frontier and resource counters for the following call.
    pub fn next_path(
        &mut self,
        graph: &Graph,
        cancellation: &dyn Cancellation,
    ) -> RuntimeResult<Option<Path>> {
        if self.finished {
            return Ok(None);
        }
        while let Some(state) = self.frontier.pop() {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            self.budget.work()?;
            self.budget.release_memory(state.memory_bytes());
            let hops = u32::try_from(state.relationships.len()).map_err(|_| {
                RuntimeError::LimitExceeded {
                    kind: crate::LimitKind::Hops,
                    limit: u64::from(self.request.max_hops),
                }
            })?;

            if hops < self.request.max_hops {
                self.expand(graph, &state, cancellation)?;
            }
            if hops >= self.request.min_hops {
                self.budget.path()?;
                return Ok(Some(state.into_path()));
            }
        }
        self.finished = true;
        Ok(None)
    }

    fn expand(
        &mut self,
        graph: &Graph,
        state: &PathState,
        cancellation: &dyn Cancellation,
    ) -> RuntimeResult<()> {
        let neighbors = graph.neighbors(
            *state.nodes.last().expect("path state always has a node"),
            self.request.direction,
            &self.request.relationship_types,
        )?;
        for neighbor in neighbors {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            self.budget.work()?;
            self.budget.edge()?;
            if !allows(
                self.request.uniqueness,
                state,
                neighbor.node,
                neighbor.relationship,
            ) {
                continue;
            }
            let mut child = state.clone();
            child.nodes.push(neighbor.node);
            child.relationships.push(neighbor.relationship);
            child.relationship_types.push(neighbor.relationship_type);
            child.total_weight = child
                .total_weight
                .checked_add(neighbor.weight)
                .ok_or(RuntimeError::CostOverflow)?;
            self.budget.node()?;
            self.budget.retain_memory(child.memory_bytes())?;
            self.frontier.push(child);
        }
        Ok(())
    }
}

impl Path {
    pub fn hop_count(&self) -> usize {
        self.relationships.len()
    }
}

#[derive(Clone)]
struct PathState {
    nodes: Vec<NodeId>,
    relationships: Vec<RelationshipId>,
    relationship_types: Vec<RelationshipTypeId>,
    total_weight: u64,
}

impl PathState {
    fn start(node: NodeId) -> Self {
        Self {
            nodes: vec![node],
            relationships: Vec::new(),
            relationship_types: Vec::new(),
            total_weight: 0,
        }
    }

    fn memory_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.nodes.len().saturating_mul(size_of::<NodeId>()))
            .saturating_add(
                self.relationships
                    .len()
                    .saturating_mul(size_of::<RelationshipId>()),
            )
            .saturating_add(
                self.relationship_types
                    .len()
                    .saturating_mul(size_of::<RelationshipTypeId>()),
            )
    }

    fn into_path(self) -> Path {
        Path {
            nodes: self.nodes,
            relationships: self.relationships,
            relationship_types: self.relationship_types,
            total_weight: self.total_weight,
        }
    }
}

enum Frontier {
    BreadthFirst(VecDeque<PathState>),
    DepthFirst(Vec<PathState>),
}

impl Frontier {
    fn new(order: TraversalOrder) -> Self {
        match order {
            TraversalOrder::BreadthFirst => Self::BreadthFirst(VecDeque::new()),
            TraversalOrder::DepthFirst => Self::DepthFirst(Vec::new()),
        }
    }

    fn push(&mut self, state: PathState) {
        match self {
            Self::BreadthFirst(frontier) => frontier.push_back(state),
            Self::DepthFirst(frontier) => frontier.push(state),
        }
    }

    fn pop(&mut self) -> Option<PathState> {
        match self {
            Self::BreadthFirst(frontier) => frontier.pop_front(),
            Self::DepthFirst(frontier) => frontier.pop(),
        }
    }
}

pub fn traverse(
    graph: &Graph,
    request: &TraversalRequest,
    limits: TraversalLimits,
    cancellation: &dyn Cancellation,
) -> RuntimeResult<Vec<Path>> {
    validate_request(graph, request)?;
    let mut budget = Budget::new(limits)?;
    budget.require_hops(request.max_hops)?;
    budget.node()?;

    let initial = PathState::start(request.start);
    budget.retain_memory(initial.memory_bytes())?;
    let mut frontier = Frontier::new(request.order);
    frontier.push(initial);
    let mut paths = Vec::new();

    while let Some(state) = frontier.pop() {
        if cancellation.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        budget.work()?;
        budget.release_memory(state.memory_bytes());
        let hops =
            u32::try_from(state.relationships.len()).map_err(|_| RuntimeError::LimitExceeded {
                kind: crate::LimitKind::Hops,
                limit: u64::from(request.max_hops),
            })?;
        if hops >= request.min_hops {
            budget.path()?;
            let result = state.clone();
            budget.retain_memory(result.memory_bytes())?;
            paths.push(result.into_path());
        }
        if hops == request.max_hops {
            continue;
        }

        let neighbors = graph.neighbors(
            *state.nodes.last().expect("path state always has a node"),
            request.direction,
            &request.relationship_types,
        )?;
        for neighbor in neighbors {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            budget.work()?;
            budget.edge()?;
            if !allows(
                request.uniqueness,
                &state,
                neighbor.node,
                neighbor.relationship,
            ) {
                continue;
            }
            let mut child = state.clone();
            child.nodes.push(neighbor.node);
            child.relationships.push(neighbor.relationship);
            child.relationship_types.push(neighbor.relationship_type);
            child.total_weight = child
                .total_weight
                .checked_add(neighbor.weight)
                .ok_or(RuntimeError::CostOverflow)?;
            budget.node()?;
            budget.retain_memory(child.memory_bytes())?;
            frontier.push(child);
        }
    }
    Ok(paths)
}

fn validate_request(graph: &Graph, request: &TraversalRequest) -> RuntimeResult<()> {
    if request.min_hops > request.max_hops {
        return Err(RuntimeError::InvalidHopRange {
            min: request.min_hops,
            max: request.max_hops,
        });
    }
    if !graph.contains_node(request.start) {
        return Err(RuntimeError::UnknownNode(request.start));
    }
    Ok(())
}

fn allows(
    uniqueness: Uniqueness,
    state: &PathState,
    node: NodeId,
    relationship: RelationshipId,
) -> bool {
    match uniqueness {
        Uniqueness::Walk => true,
        Uniqueness::Trail => !state.relationships.contains(&relationship),
        Uniqueness::Path => !state.nodes.contains(&node),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::{BuildLimits, EdgeInput, LimitKind};

    fn node(value: u64) -> NodeId {
        NodeId::new(value).unwrap()
    }

    fn relationship(value: u64) -> RelationshipId {
        RelationshipId::new(value).unwrap()
    }

    fn relationship_type(value: u32) -> RelationshipTypeId {
        RelationshipTypeId::new(value).unwrap()
    }

    fn graph() -> Graph {
        Graph::build(
            [node(1), node(2), node(3), node(4), node(5)],
            [
                edge(10, 1, 2, 1),
                edge(11, 2, 3, 1),
                edge(12, 3, 1, 1),
                edge(13, 1, 4, 2),
                edge(14, 4, 5, 2),
            ],
            BuildLimits::default(),
        )
        .unwrap()
    }

    fn edge(id: u64, source: u64, target: u64, kind: u32) -> EdgeInput {
        EdgeInput {
            relationship: relationship(id),
            source: node(source),
            target: node(target),
            relationship_type: relationship_type(kind),
            weight: None,
        }
    }

    fn request(order: TraversalOrder, uniqueness: Uniqueness) -> TraversalRequest {
        TraversalRequest {
            start: node(1),
            direction: Direction::Outgoing,
            relationship_types: vec![],
            min_hops: 1,
            max_hops: 2,
            uniqueness,
            order,
        }
    }

    #[test]
    fn breadth_first_respects_depth_and_relationship_type_filters() {
        let graph = graph();
        let mut request = request(TraversalOrder::BreadthFirst, Uniqueness::Trail);
        request.relationship_types = vec![relationship_type(1)];
        let paths = traverse(
            &graph,
            &request,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .unwrap();
        assert_eq!(
            paths
                .iter()
                .map(|path| (path.nodes.last().copied().unwrap(), path.hop_count()))
                .collect::<Vec<_>>(),
            vec![(node(2), 1), (node(3), 2)]
        );
    }

    #[test]
    fn resumable_cursor_matches_materialized_breadth_first_results() {
        let graph = graph();
        let request = request(TraversalOrder::BreadthFirst, Uniqueness::Trail);
        let expected = traverse(
            &graph,
            &request,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .unwrap();
        let mut cursor = TraversalCursor::new(&graph, request, TraversalLimits::default()).unwrap();
        let mut actual = Vec::new();
        while let Some(path) = cursor.next_path(&graph, &crate::NeverCancelled).unwrap() {
            actual.push(path);
        }
        assert_eq!(actual, expected);
        assert_eq!(
            cursor.next_path(&graph, &crate::NeverCancelled).unwrap(),
            None
        );
    }

    #[test]
    fn depth_first_and_uniqueness_modes_have_distinct_cycle_semantics() {
        let graph = graph();
        let mut trail = request(TraversalOrder::DepthFirst, Uniqueness::Trail);
        trail.max_hops = 3;
        trail.min_hops = 3;
        trail.relationship_types = vec![relationship_type(1)];
        let trail_paths = traverse(
            &graph,
            &trail,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .unwrap();
        assert_eq!(trail_paths.len(), 1);
        assert_eq!(
            trail_paths[0].nodes,
            vec![node(1), node(2), node(3), node(1)]
        );

        trail.uniqueness = Uniqueness::Path;
        assert!(traverse(
            &graph,
            &trail,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .unwrap()
        .is_empty());
    }

    #[test]
    fn walk_can_repeat_a_relationship_that_trail_rejects() {
        let graph = Graph::build(
            [node(1), node(2)],
            [edge(10, 1, 2, 1)],
            BuildLimits::default(),
        )
        .unwrap();
        let mut request = TraversalRequest {
            start: node(1),
            direction: Direction::Both,
            relationship_types: vec![],
            min_hops: 2,
            max_hops: 2,
            uniqueness: Uniqueness::Walk,
            order: TraversalOrder::BreadthFirst,
        };
        let paths = traverse(
            &graph,
            &request,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].nodes, vec![node(1), node(2), node(1)]);
        assert_eq!(paths[0].relationships, vec![relationship(10); 2]);

        request.uniqueness = Uniqueness::Trail;
        assert!(traverse(
            &graph,
            &request,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .unwrap()
        .is_empty());
    }

    #[test]
    fn zero_hop_range_returns_only_the_seed_without_reading_edges() {
        let graph = graph();
        let request = TraversalRequest {
            start: node(1),
            direction: Direction::Outgoing,
            relationship_types: vec![],
            min_hops: 0,
            max_hops: 0,
            uniqueness: Uniqueness::Trail,
            order: TraversalOrder::BreadthFirst,
        };
        let paths = traverse(
            &graph,
            &request,
            TraversalLimits {
                max_edge_visits: 0,
                max_hops: 0,
                ..TraversalLimits::default()
            },
            &crate::NeverCancelled,
        )
        .unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].nodes, vec![node(1)]);
        assert!(paths[0].relationships.is_empty());
    }

    struct CancelAfter(AtomicUsize);

    impl Cancellation for CancelAfter {
        fn is_cancelled(&self) -> bool {
            self.0.fetch_add(1, Ordering::Relaxed) >= 2
        }
    }

    #[test]
    fn cancellation_and_each_resource_limit_fail_loudly() {
        let graph = graph();
        let request = request(TraversalOrder::BreadthFirst, Uniqueness::Walk);
        assert_eq!(
            traverse(
                &graph,
                &request,
                TraversalLimits::default(),
                &CancelAfter(AtomicUsize::new(0)),
            ),
            Err(RuntimeError::Cancelled)
        );

        let checks = [
            (
                LimitKind::Nodes,
                TraversalLimits {
                    max_node_visits: 1,
                    ..TraversalLimits::default()
                },
            ),
            (
                LimitKind::Edges,
                TraversalLimits {
                    max_edge_visits: 0,
                    ..TraversalLimits::default()
                },
            ),
            (
                LimitKind::Paths,
                TraversalLimits {
                    max_paths: 0,
                    ..TraversalLimits::default()
                },
            ),
            (
                LimitKind::Work,
                TraversalLimits {
                    max_work: 1,
                    ..TraversalLimits::default()
                },
            ),
            (
                LimitKind::Memory,
                TraversalLimits {
                    max_memory_bytes: 1,
                    ..TraversalLimits::default()
                },
            ),
        ];
        for (kind, limits) in checks {
            assert!(matches!(
                traverse(&graph, &request, limits, &crate::NeverCancelled),
                Err(RuntimeError::LimitExceeded { kind: actual, .. }) if actual == kind
            ));
        }
        assert!(matches!(
            traverse(
                &graph,
                &request,
                TraversalLimits {
                    max_hops: 1,
                    ..TraversalLimits::default()
                },
                &crate::NeverCancelled,
            ),
            Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Hops,
                ..
            })
        ));
    }
}
