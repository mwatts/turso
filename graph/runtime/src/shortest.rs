// Source: https://github.com/Evokoa/pgGraph
// Revision: d689bcf2b3b52d7f878f61718be69ebcb953affc
// Path: graph/src/path_finder.rs
// License: Apache-2.0
// Adaptation: structural-adaptation
// Changes: Replaced pgGraph node stores and projection neighbors with Turso
// graph identities and owned CSR access; added shared cancellation and resource
// accounting to unweighted BFS and weighted Dijkstra traversal; added path-policy
// resolution.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::mem::size_of;

use turso_graph_ir::{NodeId, PathUniqueness, RelationshipTypeId, RoleId};

use crate::{
    limits::Budget, resolve_path_algorithm, Cancellation, Graph, Path, PathAlgorithm, PathSelector,
    RuntimeError, RuntimeResult, TraversalLimits, WeightClass,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortestPathRequest {
    pub source: NodeId,
    pub target: NodeId,
    /// The role the path departs by at every hop.
    pub from_role: RoleId,
    /// The role the path arrives by at every hop.
    pub to_role: RoleId,
    /// When true, also traverse the reverse `(to_role, from_role)` pair,
    /// unioning both directions the way `Direction::Both` used to.
    pub symmetric: bool,
    /// Empty means "every relationship type stored under this role pair".
    pub relationship_types: Vec<RelationshipTypeId>,
    pub max_hops: u32,
}

pub fn shortest_path(
    graph: &Graph,
    request: &ShortestPathRequest,
    limits: TraversalLimits,
    cancellation: &dyn Cancellation,
) -> RuntimeResult<Option<Path>> {
    validate_request(graph, request)?;
    // Unweighted single shortest path over walks. Stated rather than assumed,
    // so a future caller cannot reach this BFS with a combination the table
    // refuses.
    debug_assert_eq!(
        resolve_path_algorithm(
            PathUniqueness::Walk,
            PathSelector::Shortest,
            WeightClass::Unweighted,
            2,
            None
        ),
        Ok(PathAlgorithm::BreadthFirst)
    );
    let mut budget = Budget::new(limits)?;
    budget.require_hops(request.max_hops)?;
    let initial = Path {
        nodes: vec![request.source],
        relationships: Vec::new(),
        relationship_types: Vec::new(),
        from_roles: Vec::new(),
        to_roles: Vec::new(),
        total_weight: 0,
    };
    budget.node()?;
    budget.retain_memory(path_memory(&initial))?;
    let mut frontier = VecDeque::from([initial]);
    let mut visited = HashSet::from([request.source]);
    budget.retain_memory(size_of::<NodeId>())?;
    let pairs = graph.resolve_pairs(
        &request.relationship_types,
        request.from_role,
        request.to_role,
        request.symmetric,
    );

    while let Some(path) = frontier.pop_front() {
        if cancellation.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        budget.work()?;
        budget.release_memory(path_memory(&path));
        let current = *path.nodes.last().expect("shortest path always has a node");
        if current == request.target {
            budget.path()?;
            budget.retain_memory(path_memory(&path))?;
            return Ok(Some(path));
        }
        if path.relationships.len() as u32 == request.max_hops {
            continue;
        }
        for neighbor in graph.neighbors(current, &pairs) {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            budget.work()?;
            budget.edge()?;
            if !visited.insert(neighbor.node) {
                continue;
            }
            budget.node()?;
            budget.retain_memory(size_of::<NodeId>())?;
            let mut child = path.clone();
            child.nodes.push(neighbor.node);
            child.relationships.push(neighbor.relationship);
            child.relationship_types.push(neighbor.relationship_type);
            child.from_roles.push(neighbor.from_role);
            child.to_roles.push(neighbor.to_role);
            child.total_weight = child
                .total_weight
                .checked_add(neighbor.weight)
                .ok_or(RuntimeError::CostOverflow)?;
            budget.retain_memory(path_memory(&child))?;
            frontier.push_back(child);
        }
    }
    Ok(None)
}

struct WeightedState {
    cost: u64,
    serial: u64,
    path: Path,
}

impl PartialEq for WeightedState {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.serial == other.serial
    }
}

impl Eq for WeightedState {}

impl PartialOrd for WeightedState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeightedState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.serial.cmp(&self.serial))
    }
}

pub fn weighted_shortest_path(
    graph: &Graph,
    request: &ShortestPathRequest,
    limits: TraversalLimits,
    cancellation: &dyn Cancellation,
) -> RuntimeResult<Option<Path>> {
    validate_request(graph, request)?;
    // Non-negative weighted single shortest path over walks. Weights are `u64`,
    // so the negative row the table refuses is unreachable from this type; the
    // assertion is what makes that dependency explicit.
    debug_assert_eq!(
        resolve_path_algorithm(
            PathUniqueness::Walk,
            PathSelector::Shortest,
            WeightClass::NonNegative,
            2,
            None
        ),
        Ok(PathAlgorithm::Dijkstra)
    );
    let mut budget = Budget::new(limits)?;
    budget.require_hops(request.max_hops)?;
    let initial = Path {
        nodes: vec![request.source],
        relationships: Vec::new(),
        relationship_types: Vec::new(),
        from_roles: Vec::new(),
        to_roles: Vec::new(),
        total_weight: 0,
    };
    let mut heap = BinaryHeap::new();
    budget.node()?;
    budget.retain_memory(path_memory(&initial))?;
    heap.push(WeightedState {
        cost: 0,
        serial: 0,
        path: initial,
    });
    let mut distances = HashMap::from([((request.source, 0), 0)]);
    budget.retain_memory(size_of::<((NodeId, u32), u64)>())?;
    let mut serial = 1u64;
    let pairs = graph.resolve_pairs(
        &request.relationship_types,
        request.from_role,
        request.to_role,
        request.symmetric,
    );

    while let Some(state) = heap.pop() {
        if cancellation.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        budget.work()?;
        budget.release_memory(path_memory(&state.path));
        let current = *state
            .path
            .nodes
            .last()
            .expect("weighted path always has a node");
        let hops = state.path.relationships.len() as u32;
        if distances.get(&(current, hops)).copied() != Some(state.cost) {
            continue;
        }
        if current == request.target {
            budget.path()?;
            budget.retain_memory(path_memory(&state.path))?;
            return Ok(Some(state.path));
        }
        if hops == request.max_hops {
            continue;
        }
        for neighbor in graph.neighbors(current, &pairs) {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            budget.work()?;
            budget.edge()?;
            let cost = state
                .cost
                .checked_add(neighbor.weight)
                .ok_or(RuntimeError::CostOverflow)?;
            let key = (neighbor.node, hops + 1);
            if distances.get(&key).is_some_and(|known| *known <= cost) {
                continue;
            }
            if distances.insert(key, cost).is_none() {
                budget.retain_memory(size_of::<((NodeId, u32), u64)>())?;
            }
            budget.node()?;
            let mut path = state.path.clone();
            path.nodes.push(neighbor.node);
            path.relationships.push(neighbor.relationship);
            path.relationship_types.push(neighbor.relationship_type);
            path.from_roles.push(neighbor.from_role);
            path.to_roles.push(neighbor.to_role);
            path.total_weight = cost;
            budget.retain_memory(path_memory(&path))?;
            heap.push(WeightedState { cost, serial, path });
            serial = serial.checked_add(1).ok_or(RuntimeError::CostOverflow)?;
        }
    }
    Ok(None)
}

fn validate_request(graph: &Graph, request: &ShortestPathRequest) -> RuntimeResult<()> {
    if !graph.contains_node(request.source) {
        return Err(RuntimeError::UnknownNode(request.source));
    }
    if !graph.contains_node(request.target) {
        return Err(RuntimeError::UnknownNode(request.target));
    }
    Ok(())
}

fn path_memory(path: &Path) -> usize {
    size_of::<Path>()
        .saturating_add(path.nodes.len().saturating_mul(size_of::<NodeId>()))
        .saturating_add(
            path.relationships
                .len()
                .saturating_mul(size_of::<turso_graph_ir::RelationshipId>()),
        )
        .saturating_add(
            path.relationship_types
                .len()
                .saturating_mul(size_of::<RelationshipTypeId>()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildLimits, EdgeInput, NeverCancelled};
    use turso_graph_ir::{RelationshipId, RelationshipTypeId};

    fn node(value: u64) -> NodeId {
        NodeId::new(value).unwrap()
    }

    fn relationship(value: u64) -> RelationshipId {
        RelationshipId::new(value).unwrap()
    }

    fn kind(value: u32) -> RelationshipTypeId {
        RelationshipTypeId::new(value).unwrap()
    }

    fn role(value: u32) -> RoleId {
        RoleId::new(value).unwrap()
    }

    fn edge(id: u64, source: u64, target: u64, weight: u64) -> EdgeInput {
        EdgeInput {
            relationship: relationship(id),
            from_role: role(1),
            to_role: role(2),
            source: node(source),
            target: node(target),
            relationship_type: kind(1),
            weight: Some(weight),
        }
    }

    fn request(source: u64, target: u64) -> ShortestPathRequest {
        ShortestPathRequest {
            source: node(source),
            target: node(target),
            from_role: role(1),
            to_role: role(2),
            symmetric: false,
            relationship_types: vec![],
            max_hops: 8,
        }
    }

    #[test]
    fn shortest_path_terminates_on_cycles_and_chooses_direct_route() {
        let graph = Graph::build(
            [node(1), node(2), node(3), node(4)],
            [
                edge(10, 1, 2, 1),
                edge(11, 2, 3, 1),
                edge(12, 3, 1, 1),
                edge(13, 1, 4, 9),
                edge(14, 3, 4, 1),
            ],
            BuildLimits::default(),
        )
        .unwrap();
        let path = shortest_path(
            &graph,
            &request(1, 4),
            TraversalLimits::default(),
            &NeverCancelled,
        )
        .unwrap()
        .unwrap();
        assert_eq!(path.nodes, vec![node(1), node(4)]);
    }

    #[test]
    fn shortest_path_returns_none_for_disconnected_nodes() {
        let graph = Graph::build([node(1), node(2)], [], BuildLimits::default()).unwrap();
        assert_eq!(
            shortest_path(
                &graph,
                &request(1, 2),
                TraversalLimits::default(),
                &NeverCancelled,
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn weighted_path_prefers_lower_total_cost_and_honors_hop_limit() {
        let graph = Graph::build(
            [node(1), node(2), node(3), node(4)],
            [
                edge(10, 1, 2, 100),
                edge(11, 2, 4, 1),
                edge(12, 1, 3, 5),
                edge(13, 3, 4, 5),
            ],
            BuildLimits::default(),
        )
        .unwrap();
        let path = weighted_shortest_path(
            &graph,
            &request(1, 4),
            TraversalLimits::default(),
            &NeverCancelled,
        )
        .unwrap()
        .unwrap();
        assert_eq!(path.nodes, vec![node(1), node(3), node(4)]);
        assert_eq!(path.total_weight, 10);

        let mut request = request(1, 4);
        request.max_hops = 1;
        let path = weighted_shortest_path(
            &graph,
            &request,
            TraversalLimits::default(),
            &NeverCancelled,
        )
        .unwrap();
        assert_eq!(path, None);
    }

    #[test]
    fn the_search_entry_points_agree_with_the_policy_table() {
        // The table is only protection if the code consults it. These two
        // entry points are the whole weighted/unweighted surface today; when a
        // third arrives it must appear here.
        use crate::{resolve_path_algorithm, PathAlgorithm, PathSelector, WeightClass};
        use turso_graph_ir::PathUniqueness;

        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Walk,
                PathSelector::Shortest,
                WeightClass::Unweighted,
                2,
                None
            ),
            Ok(PathAlgorithm::BreadthFirst),
            "shortest_path is a BFS and must resolve to one"
        );
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Walk,
                PathSelector::Shortest,
                WeightClass::NonNegative,
                2,
                None
            ),
            Ok(PathAlgorithm::Dijkstra),
            "weighted_shortest_path is a Dijkstra and must resolve to one"
        );
    }

    #[test]
    fn an_unsupported_combination_becomes_a_runtime_error() {
        use crate::{resolve_path_algorithm, PathSelector, RuntimeError, WeightClass};
        use turso_graph_ir::PathUniqueness;

        let refusal = resolve_path_algorithm(
            PathUniqueness::Walk,
            PathSelector::Shortest,
            WeightClass::Negative,
            2,
            None,
        )
        .expect_err("negative-weight walks are refused");
        let error = RuntimeError::from(refusal);
        assert!(
            matches!(error, RuntimeError::UnsupportedPathCombination { .. }),
            "unexpected error: {error}"
        );
    }
}
