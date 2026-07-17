// Source: https://github.com/Evokoa/pgGraph
// Revision: d689bcf2b3b52d7f878f61718be69ebcb953affc
// Path: graph/src/safety.rs
// License: Apache-2.0
// Adaptation: structural-adaptation
// Changes: Replaced PostgreSQL errors and GUC-backed limits with typed,
// caller-owned Rust budgets and a cancellation trait.

use crate::{LimitKind, RuntimeError, RuntimeResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildLimits {
    pub max_nodes: u64,
    pub max_edges: u64,
    pub max_memory_bytes: u64,
}

impl Default for BuildLimits {
    fn default() -> Self {
        Self {
            max_nodes: 10_000_000,
            max_edges: 100_000_000,
            max_memory_bytes: 4 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraversalLimits {
    pub max_node_visits: u64,
    pub max_edge_visits: u64,
    pub max_paths: u64,
    pub max_hops: u32,
    pub max_work: u64,
    pub max_memory_bytes: u64,
}

impl Default for TraversalLimits {
    fn default() -> Self {
        Self {
            max_node_visits: 1_000_000,
            max_edge_visits: 10_000_000,
            max_paths: 100_000,
            max_hops: 64,
            max_work: 20_000_000,
            max_memory_bytes: 256 * 1024 * 1024,
        }
    }
}

pub trait Cancellation: Send + Sync {
    fn is_cancelled(&self) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NeverCancelled;

impl Cancellation for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

pub(crate) struct Budget {
    limits: TraversalLimits,
    node_visits: u64,
    edge_visits: u64,
    paths: u64,
    work: u64,
    memory_bytes: u64,
}

impl Budget {
    pub(crate) fn new(limits: TraversalLimits) -> RuntimeResult<Self> {
        Ok(Self {
            limits,
            node_visits: 0,
            edge_visits: 0,
            paths: 0,
            work: 0,
            memory_bytes: 0,
        })
    }

    pub(crate) fn require_hops(&self, hops: u32) -> RuntimeResult<()> {
        if hops > self.limits.max_hops {
            return Err(RuntimeError::LimitExceeded {
                kind: LimitKind::Hops,
                limit: u64::from(self.limits.max_hops),
            });
        }
        Ok(())
    }

    pub(crate) fn node(&mut self) -> RuntimeResult<()> {
        self.node_visits = self.node_visits.saturating_add(1);
        check(
            self.node_visits,
            self.limits.max_node_visits,
            LimitKind::Nodes,
        )
    }

    pub(crate) fn edge(&mut self) -> RuntimeResult<()> {
        self.edge_visits = self.edge_visits.saturating_add(1);
        check(
            self.edge_visits,
            self.limits.max_edge_visits,
            LimitKind::Edges,
        )
    }

    pub(crate) fn path(&mut self) -> RuntimeResult<()> {
        self.paths = self.paths.saturating_add(1);
        check(self.paths, self.limits.max_paths, LimitKind::Paths)
    }

    pub(crate) fn work(&mut self) -> RuntimeResult<()> {
        self.work = self.work.saturating_add(1);
        check(self.work, self.limits.max_work, LimitKind::Work)
    }

    pub(crate) fn retain_memory(&mut self, bytes: usize) -> RuntimeResult<()> {
        self.memory_bytes = self.memory_bytes.saturating_add(bytes as u64);
        check(
            self.memory_bytes,
            self.limits.max_memory_bytes,
            LimitKind::Memory,
        )
    }

    pub(crate) fn release_memory(&mut self, bytes: usize) {
        self.memory_bytes = self.memory_bytes.saturating_sub(bytes as u64);
    }
}

fn check(value: u64, limit: u64, kind: LimitKind) -> RuntimeResult<()> {
    if value > limit {
        Err(RuntimeError::LimitExceeded { kind, limit })
    } else {
        Ok(())
    }
}
