// Source: https://github.com/Evokoa/pgGraph
// Revision: d689bcf2b3b52d7f878f61718be69ebcb953affc
// Path: graph/src/edge_store.rs
// License: Apache-2.0
// Adaptation: structural-adaptation
// Changes: Replaced mmap/PostgreSQL-index storage with safe owned forward and
// reverse CSR arrays keyed by Turso graph-IR identities.

use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use turso_graph_ir::{Direction, NodeId, RelationshipId, RelationshipTypeId};

use crate::{BuildLimits, Cancellation, LimitKind, NeverCancelled, RuntimeError, RuntimeResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdgeInput {
    pub relationship: RelationshipId,
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
    pub weight: u64,
}

pub struct NeighborCursor {
    direction: Direction,
    forward_position: usize,
    forward_end: usize,
    reverse_position: usize,
    reverse_end: usize,
}

pub(crate) enum NeighborCursorStep {
    Neighbor(Neighbor),
    Filtered,
    Done,
}

#[derive(Clone, Copy)]
struct CsrEdge {
    node_index: u32,
    relationship: RelationshipId,
    relationship_type: RelationshipTypeId,
    weight: u64,
}

#[derive(Default)]
struct Csr {
    offsets: Vec<usize>,
    edges: Vec<CsrEdge>,
}

pub struct Graph {
    nodes: Vec<NodeId>,
    node_indexes: HashMap<NodeId, u32>,
    forward: Csr,
    reverse: Csr,
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
            let index =
                u32::try_from(node_values.len()).map_err(|_| RuntimeError::LimitExceeded {
                    kind: LimitKind::Nodes,
                    limit: u64::from(u32::MAX),
                })?;
            if node_indexes.insert(node, index).is_some() {
                return Err(RuntimeError::DuplicateNode(node));
            }
            node_values.push(node);
        }

        let mut edge_values = Vec::new();
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
            if !relationship_ids.insert(edge.relationship) {
                return Err(RuntimeError::DuplicateRelationship(edge.relationship));
            }
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

        let estimated_bytes = estimated_graph_bytes(node_values.len(), edge_values.len());
        if estimated_bytes > limits.max_memory_bytes {
            return Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Memory,
                limit: limits.max_memory_bytes,
            });
        }
        let forward = build_csr(&node_indexes, &edge_values, false, cancellation)?;
        let reverse = build_csr(&node_indexes, &edge_values, true, cancellation)?;
        Ok(Self {
            nodes: node_values,
            node_indexes,
            forward,
            reverse,
        })
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.forward.edges.len()
    }

    pub fn contains_node(&self, node: NodeId) -> bool {
        self.node_indexes.contains_key(&node)
    }

    pub fn estimated_heap_bytes(&self) -> u64 {
        estimated_graph_bytes(self.node_count(), self.edge_count())
    }

    pub fn neighbors(
        &self,
        node: NodeId,
        direction: Direction,
        relationship_types: &[RelationshipTypeId],
    ) -> RuntimeResult<Vec<Neighbor>> {
        let mut cursor = self.neighbor_cursor(node, direction)?;
        let mut neighbors = Vec::new();
        loop {
            match cursor.step(self, relationship_types) {
                NeighborCursorStep::Neighbor(neighbor) => neighbors.push(neighbor),
                NeighborCursorStep::Filtered => {}
                NeighborCursorStep::Done => break,
            }
        }
        Ok(neighbors)
    }

    pub fn neighbor_cursor(
        &self,
        node: NodeId,
        direction: Direction,
    ) -> RuntimeResult<NeighborCursor> {
        let index = *self
            .node_indexes
            .get(&node)
            .ok_or(RuntimeError::UnknownNode(node))? as usize;
        Ok(NeighborCursor {
            direction,
            forward_position: self.forward.offsets[index],
            forward_end: self.forward.offsets[index + 1],
            reverse_position: self.reverse.offsets[index],
            reverse_end: self.reverse.offsets[index + 1],
        })
    }
}

impl NeighborCursor {
    /// The caller supplies `relationship_types` on every step so the cursor
    /// never has to copy the filter; per-node cursors share the traversal's
    /// one filter allocation.
    pub(crate) fn step(
        &mut self,
        graph: &Graph,
        relationship_types: &[RelationshipTypeId],
    ) -> NeighborCursorStep {
        let edge = match self.direction {
            Direction::Outgoing => {
                take_edge(&graph.forward, &mut self.forward_position, self.forward_end)
            }
            Direction::Incoming => {
                take_edge(&graph.reverse, &mut self.reverse_position, self.reverse_end)
            }
            Direction::Both => self.take_both(graph),
        };
        let Some(edge) = edge else {
            return NeighborCursorStep::Done;
        };
        if !relationship_types.is_empty() && !relationship_types.contains(&edge.relationship_type) {
            return NeighborCursorStep::Filtered;
        }
        NeighborCursorStep::Neighbor(Neighbor {
            node: graph.nodes[edge.node_index as usize],
            relationship: edge.relationship,
            relationship_type: edge.relationship_type,
            weight: edge.weight,
        })
    }

    fn take_both(&mut self, graph: &Graph) -> Option<CsrEdge> {
        let forward = graph.forward.edges.get(self.forward_position).copied();
        let reverse = graph.reverse.edges.get(self.reverse_position).copied();
        match (
            forward.filter(|_| self.forward_position < self.forward_end),
            reverse.filter(|_| self.reverse_position < self.reverse_end),
        ) {
            (Some(forward), Some(reverse)) => {
                let forward_key = edge_key(forward);
                let reverse_key = edge_key(reverse);
                if forward_key <= reverse_key {
                    self.forward_position += 1;
                    if forward.relationship == reverse.relationship {
                        self.reverse_position += 1;
                    }
                    Some(forward)
                } else {
                    self.reverse_position += 1;
                    Some(reverse)
                }
            }
            (Some(forward), None) => {
                self.forward_position += 1;
                Some(forward)
            }
            (None, Some(reverse)) => {
                self.reverse_position += 1;
                Some(reverse)
            }
            (None, None) => None,
        }
    }
}

fn take_edge(csr: &Csr, position: &mut usize, end: usize) -> Option<CsrEdge> {
    if *position >= end {
        return None;
    }
    let edge = csr.edges[*position];
    *position += 1;
    Some(edge)
}

fn edge_key(edge: CsrEdge) -> (u32, RelationshipId, RelationshipTypeId) {
    (edge.node_index, edge.relationship, edge.relationship_type)
}

fn build_csr(
    indexes: &HashMap<NodeId, u32>,
    edges: &[EdgeInput],
    reverse: bool,
    cancellation: &dyn Cancellation,
) -> RuntimeResult<Csr> {
    let mut rows = vec![Vec::new(); indexes.len()];
    for edge in edges {
        if cancellation.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        let (row_node, adjacent_node) = if reverse {
            (edge.target, edge.source)
        } else {
            (edge.source, edge.target)
        };
        rows[indexes[&row_node] as usize].push(CsrEdge {
            node_index: indexes[&adjacent_node],
            relationship: edge.relationship,
            relationship_type: edge.relationship_type,
            weight: edge.weight.unwrap_or(1),
        });
    }
    let mut offsets = Vec::with_capacity(rows.len() + 1);
    let mut values = Vec::with_capacity(edges.len());
    offsets.push(0);
    for row in &mut rows {
        if cancellation.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        row.sort_unstable_by_key(|edge| {
            (edge.node_index, edge.relationship, edge.relationship_type)
        });
        values.append(row);
        offsets.push(values.len());
    }
    Ok(Csr {
        offsets,
        edges: values,
    })
}

fn estimated_graph_bytes(node_count: usize, edge_count: usize) -> u64 {
    let nodes = node_count.saturating_mul(size_of::<NodeId>());
    let indexes = node_count.saturating_mul(size_of::<(NodeId, u32)>() * 2);
    let offsets = node_count
        .saturating_add(1)
        .saturating_mul(size_of::<usize>())
        .saturating_mul(2);
    let edges = edge_count
        .saturating_mul(size_of::<CsrEdge>())
        .saturating_mul(2);
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

    #[test]
    fn builds_forward_and_reverse_csr_without_query_time_edge_scans() {
        let graph = Graph::build(
            [node(1), node(2), node(3)],
            [
                EdgeInput {
                    relationship: relationship(10),
                    source: node(1),
                    target: node(2),
                    relationship_type: relationship_type(1),
                    weight: None,
                },
                EdgeInput {
                    relationship: relationship(11),
                    source: node(3),
                    target: node(2),
                    relationship_type: relationship_type(2),
                    weight: Some(7),
                },
            ],
            BuildLimits::default(),
        )
        .unwrap();

        assert_eq!(
            graph.neighbors(node(2), Direction::Incoming, &[]).unwrap(),
            vec![
                Neighbor {
                    node: node(1),
                    relationship: relationship(10),
                    relationship_type: relationship_type(1),
                    weight: 1,
                },
                Neighbor {
                    node: node(3),
                    relationship: relationship(11),
                    relationship_type: relationship_type(2),
                    weight: 7,
                },
            ]
        );
    }

    #[test]
    fn rejects_invalid_endpoints_duplicates_and_build_limits() {
        let edge = EdgeInput {
            relationship: relationship(10),
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
}
