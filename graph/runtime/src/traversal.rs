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

use turso_graph_ir::{NodeId, RelationshipId, RelationshipTypeId, RoleId};

use crate::{
    limits::Budget, Cancellation, Graph, NeighborCursor, RuntimeError, RuntimeResult,
    TraversalLimits,
};

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
    /// The role the path departs by at every hop.
    pub from_role: RoleId,
    /// The role the path arrives by at every hop.
    pub to_role: RoleId,
    /// When true, also traverse the reverse `(to_role, from_role)` pair,
    /// unioning both directions the way `Direction::Both` used to.
    pub symmetric: bool,
    /// Empty means "every relationship type stored under this role pair".
    pub relationship_types: Vec<RelationshipTypeId>,
    pub min_hops: u32,
    pub max_hops: u32,
    /// When true, `max_hops` is a resource cap on an unbounded pattern
    /// (`[*]`), not part of the query's meaning: reaching it while a longer
    /// admissible path exists is an error rather than silent truncation.
    pub error_at_max_hops: bool,
    pub uniqueness: Uniqueness,
    pub order: TraversalOrder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Path {
    pub nodes: Vec<NodeId>,
    pub relationships: Vec<RelationshipId>,
    pub relationship_types: Vec<RelationshipTypeId>,
    /// The role each hop's relationship was entered by, parallel to
    /// `relationships`. A relation can appear in more than one role pair, so
    /// a path element cannot be read back without recording which one this
    /// hop used.
    pub from_roles: Vec<RoleId>,
    /// The role each hop's relationship was left by, parallel to
    /// `relationships`.
    pub to_roles: Vec<RoleId>,
    pub total_weight: u64,
}

/// Resumable traversal state used by execution adapters that cannot
/// materialize every matching path before returning the first row.
pub struct TraversalCursor {
    request: TraversalRequest,
    budget: Budget,
    frontier: Frontier,
    active: Option<ActivePath>,
    finished: bool,
}

pub enum TraversalStep {
    Path(Path),
    Pending,
    Done,
}

struct ActivePath {
    state: PathState,
    neighbors: Option<NeighborCursor>,
    emit: bool,
    /// The path already has `max_hops` hops; any admissible extension found
    /// under `error_at_max_hops` proves silent truncation and must error.
    at_cap: bool,
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
            active: None,
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
        loop {
            match self.step(graph, cancellation, 1024)? {
                TraversalStep::Path(path) => return Ok(Some(path)),
                TraversalStep::Pending => {}
                TraversalStep::Done => return Ok(None),
            }
        }
    }

    /// Perform at most `work_quantum` state or adjacency operations. A
    /// `Pending` result preserves all progress and is safe to resume.
    pub fn step(
        &mut self,
        graph: &Graph,
        cancellation: &dyn Cancellation,
        work_quantum: u64,
    ) -> RuntimeResult<TraversalStep> {
        if self.finished {
            return Ok(TraversalStep::Done);
        }
        let work_quantum = work_quantum.max(1);
        let mut work = 0;
        loop {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            if self.active.is_none() {
                if work >= work_quantum {
                    return Ok(TraversalStep::Pending);
                }
                let Some(state) = self.frontier.pop() else {
                    self.finished = true;
                    return Ok(TraversalStep::Done);
                };
                self.budget.work()?;
                work += 1;
                self.budget.release_memory(state.memory_bytes());
                let hops = u32::try_from(state.relationships.len()).map_err(|_| {
                    RuntimeError::LimitExceeded {
                        kind: crate::LimitKind::Hops,
                        limit: u64::from(self.request.max_hops),
                    }
                })?;
                let at_cap = hops >= self.request.max_hops;
                let expand = !at_cap || self.request.error_at_max_hops;
                let neighbors = expand.then(|| {
                    let pairs = graph.resolve_pairs(
                        &self.request.relationship_types,
                        self.request.from_role,
                        self.request.to_role,
                        self.request.symmetric,
                    );
                    graph.neighbor_cursor(
                        *state.nodes.last().expect("path state always has a node"),
                        &pairs,
                    )
                });
                self.active = Some(ActivePath {
                    state,
                    neighbors,
                    emit: hops >= self.request.min_hops,
                    at_cap,
                });
            }

            let active = self.active.as_mut().expect("active path was initialized");
            if let Some(neighbors) = &mut active.neighbors {
                if work >= work_quantum {
                    return Ok(TraversalStep::Pending);
                }
                if cancellation.is_cancelled() {
                    return Err(RuntimeError::Cancelled);
                }
                self.budget.work()?;
                work += 1;
                match neighbors.next() {
                    None => {
                        active.neighbors = None;
                        continue;
                    }
                    Some(neighbor) => {
                        self.budget.edge()?;
                        if !allows(
                            self.request.uniqueness,
                            &active.state,
                            neighbor.node,
                            neighbor.relationship,
                        ) {
                            continue;
                        }
                        if active.at_cap {
                            // An admissible edge beyond the cap exists, so
                            // results would be silently incomplete.
                            return Err(RuntimeError::LimitExceeded {
                                kind: crate::LimitKind::Hops,
                                limit: u64::from(self.request.max_hops),
                            });
                        }
                        let mut child = active.state.clone();
                        child.nodes.push(neighbor.node);
                        child.relationships.push(neighbor.relationship);
                        child.relationship_types.push(neighbor.relationship_type);
                        child.from_roles.push(neighbor.from_role);
                        child.to_roles.push(neighbor.to_role);
                        child.total_weight = child
                            .total_weight
                            .checked_add(neighbor.weight)
                            .ok_or(RuntimeError::CostOverflow)?;
                        self.budget.node()?;
                        self.budget.retain_memory(child.memory_bytes())?;
                        self.frontier.push(child);
                        continue;
                    }
                }
            }
            let active = self.active.take().expect("completed active path exists");
            if active.emit {
                self.budget.path()?;
                return Ok(TraversalStep::Path(active.state.into_path()));
            }
        }
    }
}

impl Path {
    pub fn hop_count(&self) -> usize {
        self.relationships.len()
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
            .saturating_add(self.from_roles.len().saturating_mul(size_of::<RoleId>()))
            .saturating_add(self.to_roles.len().saturating_mul(size_of::<RoleId>()))
    }
}

#[derive(Clone)]
struct PathState {
    nodes: Vec<NodeId>,
    relationships: Vec<RelationshipId>,
    relationship_types: Vec<RelationshipTypeId>,
    from_roles: Vec<RoleId>,
    to_roles: Vec<RoleId>,
    total_weight: u64,
}

impl PathState {
    fn start(node: NodeId) -> Self {
        Self {
            nodes: vec![node],
            relationships: Vec::new(),
            relationship_types: Vec::new(),
            from_roles: Vec::new(),
            to_roles: Vec::new(),
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
            .saturating_add(self.from_roles.len().saturating_mul(size_of::<RoleId>()))
            .saturating_add(self.to_roles.len().saturating_mul(size_of::<RoleId>()))
    }

    fn into_path(self) -> Path {
        Path {
            nodes: self.nodes,
            relationships: self.relationships,
            relationship_types: self.relationship_types,
            from_roles: self.from_roles,
            to_roles: self.to_roles,
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

/// Materialize every matching path by driving a [`TraversalCursor`], so the
/// eager and resumable APIs share one expansion implementation.
pub fn traverse(
    graph: &Graph,
    request: &TraversalRequest,
    limits: TraversalLimits,
    cancellation: &dyn Cancellation,
) -> RuntimeResult<Vec<Path>> {
    let mut cursor = TraversalCursor::new(graph, request.clone(), limits)?;
    let mut paths = Vec::new();
    while let Some(path) = cursor.next_path(graph, cancellation)? {
        // Unlike streaming callers, this API retains every result, so the
        // accumulated paths stay accounted against the traversal budget.
        cursor.budget.retain_memory(path.memory_bytes())?;
        paths.push(path);
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

    fn role(value: u32) -> RoleId {
        RoleId::new(value).unwrap()
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

    /// One directed role pair (role 1 -> role 2), the shape a `direction:
    /// Outgoing` request used to read.
    fn edge(id: u64, source: u64, target: u64, kind: u32) -> EdgeInput {
        EdgeInput {
            relationship: relationship(id),
            from_role: role(1),
            to_role: role(2),
            source: node(source),
            target: node(target),
            relationship_type: relationship_type(kind),
            weight: None,
        }
    }

    fn request(order: TraversalOrder, uniqueness: Uniqueness) -> TraversalRequest {
        TraversalRequest {
            start: node(1),
            from_role: role(1),
            to_role: role(2),
            symmetric: false,
            relationship_types: vec![],
            min_hops: 1,
            max_hops: 2,
            error_at_max_hops: false,
            uniqueness,
            order,
        }
    }

    #[test]
    fn error_at_max_hops_rejects_silent_truncation() {
        // 1 -> 4 -> 5 has an admissible edge past a 1-hop cap. With
        // error_at_max_hops the cap is a resource limit on an unbounded
        // pattern, so hitting it with a longer real path must error rather
        // than silently drop paths; without it the cap is query semantics
        // and truncation is correct.
        let graph = graph();
        let mut capped = request(TraversalOrder::BreadthFirst, Uniqueness::Trail);
        capped.relationship_types = vec![relationship_type(2)];
        capped.max_hops = 1;
        assert!(traverse(
            &graph,
            &capped,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .is_ok());

        capped.error_at_max_hops = true;
        assert!(matches!(
            traverse(
                &graph,
                &capped,
                TraversalLimits::default(),
                &crate::NeverCancelled,
            ),
            Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Hops,
                limit: 1,
            })
        ));

        // A cap deep enough to hold every path stays error-free.
        capped.max_hops = 2;
        assert!(traverse(
            &graph,
            &capped,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .is_ok());
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
    fn work_quantum_bounds_no_result_progress_and_resumes_exactly() {
        let nodes = (1..=102).map(node).collect::<Vec<_>>();
        let edges = (2..=102)
            .map(|target| edge(target + 100, 1, target, 1))
            .collect::<Vec<_>>();
        let graph = Graph::build(nodes, edges, BuildLimits::default()).unwrap();
        let request = TraversalRequest {
            start: node(1),
            from_role: role(1),
            to_role: role(2),
            symmetric: false,
            relationship_types: vec![],
            min_hops: 2,
            max_hops: 2,
            error_at_max_hops: false,
            uniqueness: Uniqueness::Trail,
            order: TraversalOrder::BreadthFirst,
        };
        let mut cursor = TraversalCursor::new(&graph, request, TraversalLimits::default()).unwrap();
        let mut pending = 0;
        loop {
            match cursor.step(&graph, &crate::NeverCancelled, 7).unwrap() {
                TraversalStep::Pending => pending += 1,
                TraversalStep::Done => break,
                TraversalStep::Path(path) => panic!("unexpected path {path:?}"),
            }
        }
        assert!(pending >= 20, "fanout must be split across many quanta");
        assert!(matches!(
            cursor.step(&graph, &crate::NeverCancelled, 7).unwrap(),
            TraversalStep::Done
        ));
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
    fn a_path_records_the_role_pair_each_hop_traversed() {
        // A relation with more than one role pair cannot be read back from a
        // path unless each hop remembers which pair it took: a hop to node 2
        // and a hop to node 3 share nothing but their source node and must
        // not be confused for each other on the resulting path.
        let graph = Graph::build(
            [node(1), node(2), node(3)],
            [
                EdgeInput {
                    relationship: relationship(1),
                    from_role: role(1),
                    to_role: role(2),
                    source: node(1),
                    target: node(2),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
                EdgeInput {
                    relationship: relationship(2),
                    from_role: role(1),
                    to_role: role(3),
                    source: node(1),
                    target: node(3),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
            ],
            BuildLimits::default(),
        )
        .unwrap();

        let mut request = TraversalRequest {
            start: node(1),
            from_role: role(1),
            to_role: role(2),
            symmetric: false,
            relationship_types: vec![],
            min_hops: 1,
            max_hops: 1,
            error_at_max_hops: false,
            uniqueness: Uniqueness::Trail,
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
        assert_eq!(paths[0].from_roles, vec![role(1)]);
        assert_eq!(paths[0].to_roles, vec![role(2)]);

        request.to_role = role(3);
        let paths = traverse(
            &graph,
            &request,
            TraversalLimits::default(),
            &crate::NeverCancelled,
        )
        .unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].from_roles, vec![role(1)]);
        assert_eq!(paths[0].to_roles, vec![role(3)]);
    }

    #[test]
    fn walk_can_repeat_a_relationship_that_trail_rejects() {
        // A symmetric request over role pair (1, 2) merges it with its
        // reverse, so both directions of the one physical relationship must
        // be present: `Direction::Both` used to read one shared reverse CSR,
        // which a role-pair-keyed graph no longer builds implicitly.
        let graph = Graph::build(
            [node(1), node(2)],
            [
                edge(10, 1, 2, 1),
                EdgeInput {
                    relationship: relationship(10),
                    from_role: role(2),
                    to_role: role(1),
                    source: node(2),
                    target: node(1),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
            ],
            BuildLimits::default(),
        )
        .unwrap();
        let mut request = TraversalRequest {
            start: node(1),
            from_role: role(1),
            to_role: role(2),
            symmetric: true,
            relationship_types: vec![],
            min_hops: 2,
            max_hops: 2,
            error_at_max_hops: false,
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
            from_role: role(1),
            to_role: role(2),
            symmetric: false,
            relationship_types: vec![],
            min_hops: 0,
            max_hops: 0,
            error_at_max_hops: false,
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
