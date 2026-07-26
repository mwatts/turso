// Source: https://github.com/Evokoa/pgGraph
// Revision: d689bcf2b3b52d7f878f61718be69ebcb953affc
// Path: graph/src/edge_store.rs
// License: Apache-2.0
// Adaptation: structural-adaptation
// Changes: Replaced mmap/PostgreSQL-index storage with safe owned CSR arrays
// keyed by Turso graph-IR identities, one per ordered role pair traversed
// rather than a single forward/reverse pair.

use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use turso_graph_ir::{NodeId, RelationshipId, RelationshipTypeId, RoleId};

use crate::{BuildLimits, Cancellation, LimitKind, NeverCancelled, RuntimeError, RuntimeResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdgeInput {
    pub relationship: RelationshipId,
    pub from_role: RoleId,
    pub to_role: RoleId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship_type: RelationshipTypeId,
    pub weight: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Neighbor {
    pub node: NodeId,
    pub relationship: RelationshipId,
    pub relationship_type: RelationshipTypeId,
    pub from_role: RoleId,
    pub to_role: RoleId,
    pub weight: u64,
}

#[derive(Clone, Copy)]
struct CsrEdge {
    node_index: u32,
    relationship: RelationshipId,
    weight: u64,
}

/// One node's adjacency for a single ordered `(relationship_type, from_role,
/// to_role)` triple.
#[derive(Default)]
struct Csr {
    offsets: Vec<usize>,
    edges: Vec<CsrEdge>,
}

/// Adjacency keyed by the ordered role pair it traverses.
///
/// A relation with k roles exposes k*(k-1) directed pairs. For k = 2 that is
/// exactly the forward and reverse CSR this replaced, so a binary graph
/// builds the same two structures it always did; `neighbor_cursor` merges
/// however many pairs a caller names (a single directed hop, or a pair plus
/// its reverse for a symmetric one, or several relationship types) with the
/// same self-loop dedup a two-lane forward/reverse merge always needed.
pub struct Graph {
    nodes: Vec<NodeId>,
    node_indexes: HashMap<NodeId, usize>,
    adjacency: HashMap<(RelationshipTypeId, RoleId, RoleId), Csr>,
    /// Count of distinct physical relationships, independent of how many
    /// role-pair buckets each one contributed an edge to.
    relationship_count: usize,
}

impl Graph {
    pub fn build(
        nodes: impl IntoIterator<Item = NodeId>,
        edges: impl IntoIterator<Item = EdgeInput>,
        limits: BuildLimits,
    ) -> RuntimeResult<Self> {
        Self::build_cancellable(nodes, edges, limits, &NeverCancelled)
    }

    pub fn build_cancellable(
        nodes: impl IntoIterator<Item = NodeId>,
        edges: impl IntoIterator<Item = EdgeInput>,
        limits: BuildLimits,
        cancellation: &dyn Cancellation,
    ) -> RuntimeResult<Self> {
        let mut node_values = Vec::new();
        let mut node_indexes = HashMap::new();
        for node in nodes {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            if node_values.len() as u64 >= limits.max_nodes {
                return Err(RuntimeError::LimitExceeded {
                    kind: LimitKind::Nodes,
                    limit: limits.max_nodes,
                });
            }
            // CsrEdge stores node_index as u32 for compactness, so the index
            // must fit even though the public map is keyed by usize.
            let index =
                u32::try_from(node_values.len()).map_err(|_| RuntimeError::LimitExceeded {
                    kind: LimitKind::Nodes,
                    limit: u64::from(u32::MAX),
                })?;
            if node_indexes.insert(node, index as usize).is_some() {
                return Err(RuntimeError::DuplicateNode(node));
            }
            node_values.push(node);
        }

        let mut edge_values = Vec::new();
        let mut seen_edges = HashSet::new();
        let mut relationship_ids = HashSet::new();
        for edge in edges {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            if edge_values.len() as u64 >= limits.max_edges {
                return Err(RuntimeError::LimitExceeded {
                    kind: LimitKind::Edges,
                    limit: limits.max_edges,
                });
            }
            // One physical relationship legitimately contributes one edge per
            // ordered role pair for every (from-player, to-player) combination
            // participating in it: k*(k-1) of them when every role is
            // single-valued, more when a `Many` role's several players each
            // pair with the other side. So the same relationship id, and even
            // the same (relationship, from_role, to_role) triple, recurring is
            // expected. The same edge -- identical relationship, roles, source,
            // AND target -- recurring is not: that always means the snapshot
            // builder pushed the identical pairing twice.
            if !seen_edges.insert((
                edge.relationship,
                edge.from_role,
                edge.to_role,
                edge.source,
                edge.target,
            )) {
                return Err(RuntimeError::DuplicateRelationship(edge.relationship));
            }
            relationship_ids.insert(edge.relationship);
            if !node_indexes.contains_key(&edge.source) {
                return Err(RuntimeError::UnknownEndpoint {
                    relationship: edge.relationship,
                    endpoint: "source",
                    node: edge.source,
                });
            }
            if !node_indexes.contains_key(&edge.target) {
                return Err(RuntimeError::UnknownEndpoint {
                    relationship: edge.relationship,
                    endpoint: "target",
                    node: edge.target,
                });
            }
            edge_values.push(edge);
        }

        let adjacency = build_adjacency(&node_indexes, &edge_values, cancellation)?;
        let total_edges = adjacency.values().map(|csr| csr.edges.len()).sum();
        let estimated_bytes =
            estimated_graph_bytes(node_values.len(), adjacency.len(), total_edges);
        if estimated_bytes > limits.max_memory_bytes {
            return Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Memory,
                limit: limits.max_memory_bytes,
            });
        }
        Ok(Self {
            nodes: node_values,
            node_indexes,
            adjacency,
            relationship_count: relationship_ids.len(),
        })
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Count of distinct physical relationships. Not the number of stored
    /// adjacency rows: a binary relation stores two rows (one per direction)
    /// per relationship, and an n-ary one stores more, but each relationship
    /// counts once here regardless.
    pub fn edge_count(&self) -> usize {
        self.relationship_count
    }

    pub fn contains_node(&self, node: NodeId) -> bool {
        self.node_indexes.contains_key(&node)
    }

    pub fn estimated_heap_bytes(&self) -> u64 {
        let total_edges = self.adjacency.values().map(|csr| csr.edges.len()).sum();
        estimated_graph_bytes(self.node_count(), self.adjacency.len(), total_edges)
    }

    /// Every relationship type for which some relationship in this graph
    /// traverses the given ordered role pair. Used to resolve "any
    /// relationship type" traversal requests against role-pair-keyed
    /// adjacency, where the type is now part of the bucket key rather than a
    /// post-hoc filter over one shared CSR.
    pub(crate) fn relationship_types_for_pair(
        &self,
        from_role: RoleId,
        to_role: RoleId,
    ) -> impl Iterator<Item = RelationshipTypeId> + '_ {
        self.adjacency
            .keys()
            .filter(move |&&(_, from, to)| from == from_role && to == to_role)
            .map(|&(relationship_type, _, _)| relationship_type)
    }

    /// Resolve a traversal or shortest-path request's relationship-type
    /// filter and role pair into the concrete `(type, from_role, to_role)`
    /// triples [`neighbor_cursor`](Self::neighbor_cursor) reads. An empty
    /// `relationship_types` means "any type stored under this pair";
    /// `symmetric` additionally includes the reverse pair, unioning both
    /// directions the way `Direction::Both` used to.
    pub fn resolve_pairs(
        &self,
        relationship_types: &[RelationshipTypeId],
        from_role: RoleId,
        to_role: RoleId,
        symmetric: bool,
    ) -> Vec<(RelationshipTypeId, RoleId, RoleId)> {
        let mut role_pairs = vec![(from_role, to_role)];
        if symmetric {
            role_pairs.push((to_role, from_role));
        }
        role_pairs
            .into_iter()
            .flat_map(|(from, to)| {
                if relationship_types.is_empty() {
                    self.relationship_types_for_pair(from, to)
                        .map(|relationship_type| (relationship_type, from, to))
                        .collect::<Vec<_>>()
                } else {
                    relationship_types
                        .iter()
                        .map(|&relationship_type| (relationship_type, from, to))
                        .collect::<Vec<_>>()
                }
            })
            .collect()
    }

    /// Materialize every neighbor reachable from `node` over the given
    /// ordered `(relationship_type, from_role, to_role)` triples, merged and
    /// deduplicated as [`neighbor_cursor`](Self::neighbor_cursor) would
    /// stream them.
    pub fn neighbors(
        &self,
        node: NodeId,
        pairs: &[(RelationshipTypeId, RoleId, RoleId)],
    ) -> Vec<Neighbor> {
        self.neighbor_cursor(node, pairs).collect()
    }

    /// A streaming cursor over every neighbor reachable from `node` across
    /// the given ordered role-pair triples. A triple absent from the graph
    /// (unknown type, unknown role pair, or an unknown node) contributes an
    /// empty lane rather than an error: the traversal engines that drive
    /// this validate the start node once, up front, and only ever step from
    /// nodes already known to exist.
    ///
    /// Each lane resolves and copies its slice of adjacency up front rather
    /// than borrowing from `self`: callers persist a cursor alongside the
    /// snapshot it was built from (the same struct holds both), which a
    /// borrow of one from the other cannot express safely. The copy is
    /// bounded by `node`'s out-degree for the requested pairs, not by graph
    /// size, so it does not reintroduce a query-time edge scan.
    pub fn neighbor_cursor(
        &self,
        node: NodeId,
        pairs: &[(RelationshipTypeId, RoleId, RoleId)],
    ) -> NeighborCursor {
        let Some(index) = self.node_indexes.get(&node).copied() else {
            return NeighborCursor { lanes: Vec::new() };
        };
        let lanes = pairs
            .iter()
            .filter_map(|&(relationship_type, from_role, to_role)| {
                let csr = self
                    .adjacency
                    .get(&(relationship_type, from_role, to_role))?;
                let start = csr.offsets.get(index).copied().unwrap_or(0);
                let end = csr.offsets.get(index + 1).copied().unwrap_or(start);
                let edges = csr.edges[start..end]
                    .iter()
                    .map(|edge| ResolvedEdge {
                        node: self.nodes[edge.node_index as usize],
                        node_index: edge.node_index,
                        relationship: edge.relationship,
                        weight: edge.weight,
                    })
                    .collect();
                Some(Lane {
                    edges,
                    position: 0,
                    relationship_type,
                    from_role,
                    to_role,
                })
            })
            .collect();
        NeighborCursor { lanes }
    }
}

#[derive(Clone, Copy)]
struct ResolvedEdge {
    node: NodeId,
    node_index: u32,
    relationship: RelationshipId,
    weight: u64,
}

struct Lane {
    edges: Vec<ResolvedEdge>,
    position: usize,
    relationship_type: RelationshipTypeId,
    from_role: RoleId,
    to_role: RoleId,
}

/// Streams neighbors from one or more role-pair lanes in `node_index` order,
/// generalizing the old two-lane forward/reverse merge to however many
/// lanes a caller names. Lanes tied on the exact same `(node_index,
/// relationship)` are a self-loop under a pair traversed alongside its
/// reverse (or the same relationship visible through more than one
/// requested pair): every tied lane advances, but exactly one [`Neighbor`]
/// is emitted, matching the old merge's self-loop dedup.
pub struct NeighborCursor {
    lanes: Vec<Lane>,
}

impl Iterator for NeighborCursor {
    type Item = Neighbor;

    fn next(&mut self) -> Option<Neighbor> {
        let mut min_key: Option<(u32, RelationshipId)> = None;
        for lane in &self.lanes {
            let Some(edge) = lane.edges.get(lane.position) else {
                continue;
            };
            let key = (edge.node_index, edge.relationship);
            let take = match min_key {
                None => true,
                Some(current) => key < current,
            };
            if take {
                min_key = Some(key);
            }
        }
        let (node_index, relationship) = min_key?;
        let mut result = None;
        for lane in &mut self.lanes {
            let Some(edge) = lane.edges.get(lane.position).copied() else {
                continue;
            };
            if edge.node_index != node_index || edge.relationship != relationship {
                continue;
            }
            if result.is_none() {
                result = Some(Neighbor {
                    node: edge.node,
                    relationship: edge.relationship,
                    relationship_type: lane.relationship_type,
                    from_role: lane.from_role,
                    to_role: lane.to_role,
                    weight: edge.weight,
                });
            }
            lane.position += 1;
        }
        result
    }
}

fn build_adjacency(
    indexes: &HashMap<NodeId, usize>,
    edges: &[EdgeInput],
    cancellation: &dyn Cancellation,
) -> RuntimeResult<HashMap<(RelationshipTypeId, RoleId, RoleId), Csr>> {
    let mut rows_by_pair: HashMap<(RelationshipTypeId, RoleId, RoleId), Vec<Vec<CsrEdge>>> =
        HashMap::new();
    for edge in edges {
        if cancellation.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        let key = (edge.relationship_type, edge.from_role, edge.to_role);
        let rows = rows_by_pair
            .entry(key)
            .or_insert_with(|| vec![Vec::new(); indexes.len()]);
        rows[indexes[&edge.source]].push(CsrEdge {
            node_index: indexes[&edge.target] as u32,
            relationship: edge.relationship,
            weight: edge.weight.unwrap_or(1),
        });
    }

    let mut adjacency = HashMap::with_capacity(rows_by_pair.len());
    for (key, mut rows) in rows_by_pair {
        if cancellation.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        let mut offsets = Vec::with_capacity(rows.len() + 1);
        let mut values = Vec::new();
        offsets.push(0);
        for row in &mut rows {
            if cancellation.is_cancelled() {
                return Err(RuntimeError::Cancelled);
            }
            row.sort_unstable_by_key(|edge| (edge.node_index, edge.relationship));
            values.append(row);
            offsets.push(values.len());
        }
        adjacency.insert(
            key,
            Csr {
                offsets,
                edges: values,
            },
        );
    }
    Ok(adjacency)
}

fn estimated_graph_bytes(node_count: usize, bucket_count: usize, edge_count: usize) -> u64 {
    let nodes = node_count.saturating_mul(size_of::<NodeId>());
    let indexes = node_count.saturating_mul(size_of::<(NodeId, usize)>());
    let offsets = bucket_count
        .saturating_mul(node_count.saturating_add(1))
        .saturating_mul(size_of::<usize>());
    let edges = edge_count.saturating_mul(size_of::<CsrEdge>());
    nodes
        .saturating_add(indexes)
        .saturating_add(offsets)
        .saturating_add(edges) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// A binary "KNOWS"-shaped graph: one relationship, roles 1 ("start")
    /// and 2 ("end"), stored as both directed pairs (the same two rows the
    /// old forward/reverse CSR always built).
    fn binary_graph() -> Graph {
        Graph::build(
            [node(1), node(2)],
            [
                EdgeInput {
                    relationship: relationship(10),
                    from_role: role(1),
                    to_role: role(2),
                    source: node(1),
                    target: node(2),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
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
        .unwrap()
    }

    /// A ternary relationship over three roles (1 = scribe, 2 = text, 3 =
    /// folio), stored as all six ordered pairs the way a snapshot builder
    /// would emit them.
    fn ternary_graph() -> Graph {
        let pairs = [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)];
        let edges = pairs.map(|(from, to)| EdgeInput {
            relationship: relationship(1),
            from_role: role(from),
            to_role: role(to),
            source: node(from as u64),
            target: node(to as u64),
            relationship_type: relationship_type(1),
            weight: None,
        });
        Graph::build([node(1), node(2), node(3)], edges, BuildLimits::default()).unwrap()
    }

    #[test]
    fn adjacency_is_keyed_by_the_role_pair_it_was_built_from() {
        // A single forward/reverse pair cannot hold a ternary relation's six
        // directed pairs; merging them would let a scribe->text hop return a
        // folio.
        let graph = ternary_graph();
        let scribe_to_text = graph
            .neighbors(node(1), &[(relationship_type(1), role(1), role(2))])
            .into_iter()
            .map(|neighbor| neighbor.node)
            .collect::<Vec<_>>();
        assert_eq!(scribe_to_text, vec![node(2)]);
        let scribe_to_folio = graph
            .neighbors(node(1), &[(relationship_type(1), role(1), role(3))])
            .into_iter()
            .map(|neighbor| neighbor.node)
            .collect::<Vec<_>>();
        assert_eq!(scribe_to_folio, vec![node(3)]);
    }

    #[test]
    fn a_two_role_graph_has_exactly_the_two_pairs_it_had_as_forward_and_reverse() {
        let graph = binary_graph();
        assert_eq!(graph.adjacency.len(), 2, "one per direction, as before");
    }

    #[test]
    fn neighbors_carry_the_role_pair_the_hop_traversed() {
        let graph = binary_graph();
        let neighbors = graph.neighbors(node(1), &[(relationship_type(1), role(1), role(2))]);
        assert_eq!(
            neighbors,
            vec![Neighbor {
                node: node(2),
                relationship: relationship(10),
                relationship_type: relationship_type(1),
                from_role: role(1),
                to_role: role(2),
                weight: 1,
            }]
        );
    }

    #[test]
    fn merging_a_pair_with_its_reverse_dedups_a_self_loop_but_not_distinct_relationships() {
        // A symmetric traversal merges (from, to) with (to, from). A
        // relationship whose source and target are the same node then ties
        // in both lanes on the exact same (node_index, relationship) key and
        // must be emitted once, exactly as the old take_both merge did. A
        // distinct relationship stored in only one of the two lanes is not a
        // tie and must still be kept, once.
        let graph = Graph::build(
            [node(1), node(2)],
            [
                // A self-loop on node 1: both directions of the pair land on
                // the same node, so the two lanes tie.
                EdgeInput {
                    relationship: relationship(20),
                    from_role: role(1),
                    to_role: role(2),
                    source: node(1),
                    target: node(1),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
                EdgeInput {
                    relationship: relationship(20),
                    from_role: role(2),
                    to_role: role(1),
                    source: node(1),
                    target: node(1),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
                // A distinct, non-looping relationship stored in only the
                // forward lane: nothing to tie against, so it must survive.
                EdgeInput {
                    relationship: relationship(21),
                    from_role: role(1),
                    to_role: role(2),
                    source: node(1),
                    target: node(2),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
            ],
            BuildLimits::default(),
        )
        .unwrap();
        let neighbors = graph.neighbors(
            node(1),
            &[
                (relationship_type(1), role(1), role(2)),
                (relationship_type(1), role(2), role(1)),
            ],
        );
        let mut relationships = neighbors
            .iter()
            .map(|neighbor| neighbor.relationship)
            .collect::<Vec<_>>();
        relationships.sort();
        assert_eq!(
            relationships,
            vec![relationship(20), relationship(21)],
            "the self-loop is emitted exactly once despite tying in both lanes; \
             the distinct relationship is unaffected"
        );
    }

    #[test]
    fn an_unknown_node_yields_an_empty_cursor_rather_than_an_error() {
        // The traversal and shortest-path engines validate the start node
        // once, up front, and only ever step from nodes already known to
        // exist; a missing lane is therefore a normal "nothing here", not a
        // fault.
        let graph = binary_graph();
        assert!(graph
            .neighbors(node(99), &[(relationship_type(1), role(1), role(2))])
            .is_empty());
    }

    #[test]
    fn rejects_invalid_endpoints_duplicates_and_build_limits() {
        let edge = EdgeInput {
            relationship: relationship(10),
            from_role: role(1),
            to_role: role(2),
            source: node(1),
            target: node(2),
            relationship_type: relationship_type(1),
            weight: None,
        };
        assert!(matches!(
            Graph::build([node(1)], [edge], BuildLimits::default()),
            Err(RuntimeError::UnknownEndpoint {
                endpoint: "target",
                ..
            })
        ));
        assert!(matches!(
            Graph::build([node(1), node(1)], [], BuildLimits::default()),
            Err(RuntimeError::DuplicateNode(_))
        ));
        assert!(matches!(
            Graph::build([node(1), node(2)], [edge, edge], BuildLimits::default()),
            Err(RuntimeError::DuplicateRelationship(_))
        ));
        assert!(matches!(
            Graph::build(
                [node(1), node(2)],
                [edge],
                BuildLimits {
                    max_edges: 0,
                    ..BuildLimits::default()
                }
            ),
            Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Edges,
                ..
            })
        ));
    }

    #[test]
    fn resolve_pairs_defaults_to_every_type_stored_under_the_pair_and_symmetric_adds_the_reverse() {
        let graph = binary_graph();
        assert_eq!(
            graph.resolve_pairs(&[], role(1), role(2), false),
            vec![(relationship_type(1), role(1), role(2))]
        );
        let mut symmetric = graph.resolve_pairs(&[], role(1), role(2), true);
        symmetric.sort();
        assert_eq!(
            symmetric,
            vec![
                (relationship_type(1), role(1), role(2)),
                (relationship_type(1), role(2), role(1)),
            ]
        );
        assert_eq!(
            graph.resolve_pairs(&[relationship_type(9)], role(1), role(2), false),
            vec![(relationship_type(9), role(1), role(2))],
            "an explicit type list is used as-is, even if the pair does not exist"
        );
    }

    #[test]
    fn edge_count_is_the_physical_relationship_count_not_the_stored_row_count() {
        // A binary relationship stores two rows (one per direction) but is
        // one relationship; a ternary one stores six but is still one.
        assert_eq!(binary_graph().edge_count(), 1);
        assert_eq!(ternary_graph().edge_count(), 1);
    }
}
