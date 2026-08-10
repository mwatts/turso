//! Cost and row estimates for `__turso_graph_expand`.
//!
//! The expand virtual table's `best_index` cannot see live graph degree, so the
//! model uses hop bounds, relationship-type count, uniqueness, and the
//! `max_paths` budget. Session/runtime limits and outer `LIMIT` feed those
//! inputs at lower time.
//!
//! Constraint RHS values are not available to `best_index` (see
//! `turso_ext::ConstraintInfo`), so lowering records per-compile estimates in a
//! thread-local site list that `best_index` reads. Core then scales
//! `IndexInfo::estimated_cost` / `estimated_rows` by outer cardinality.

use std::cell::RefCell;

/// Default branching factor when degree statistics are unavailable.
const DEFAULT_BRANCHING: f64 = 4.0;

thread_local! {
    /// Estimates for each `__turso_graph_expand` lowered on this thread during
    /// the current `lower_relational` call, in encounter order.
    static EXPAND_SITES: RefCell<Vec<ExpandCostEstimate>> = RefCell::new(Vec::new());
}

/// Clear and prepare for a new lower of one Cypher plan.
pub fn begin_lower_estimates() {
    EXPAND_SITES.with(|sites| sites.borrow_mut().clear());
}

/// Record one expand site after its hop bounds and path budget are known.
pub fn record_expand_estimate(estimate: ExpandCostEstimate) {
    EXPAND_SITES.with(|sites| sites.borrow_mut().push(estimate));
}

/// Estimate used by `best_index` for this compile's expand site(s).
///
/// One site: that estimate. Several sites: conservative envelope (max rows,
/// sum of costs) so join-order search that re-enters `best_index` stays safe.
/// No sites (raw SQL expand in tests): mid-range fallback.
pub fn planner_expand_estimate() -> ExpandCostEstimate {
    EXPAND_SITES.with(|sites| {
        let sites = sites.borrow();
        match sites.as_slice() {
            [] => estimate_expand(1, 3, 1, "trail", 100_000),
            [one] => *one,
            many => ExpandCostEstimate {
                estimated_rows: many.iter().map(|e| e.estimated_rows).max().unwrap_or(1),
                estimated_cost: many
                    .iter()
                    .map(|e| e.estimated_cost)
                    .sum::<f64>()
                    .max(0.001),
            },
        }
    })
}

/// Hard ceiling so a bad hop span cannot report absurd planner cardinalities.
const MAX_ESTIMATED_ROWS: u64 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExpandCostEstimate {
    pub estimated_rows: u64,
    pub estimated_cost: f64,
}

/// Estimate expand output for one start-node invocation.
///
/// - `min_hops` / `max_hops`: pattern hop bounds (inclusive).
/// - `type_count`: number of relationship types in the filter; `0` means every
///   type stored for the role pair (no type filter).
/// - `uniqueness`: `"walk"`, `"trail"`, or `"path"` (other values treated as walk).
/// - `max_paths`: runtime path budget (after session + outer LIMIT caps).
pub fn estimate_expand(
    min_hops: u32,
    max_hops: u32,
    type_count: u32,
    uniqueness: &str,
    max_paths: u64,
) -> ExpandCostEstimate {
    let min_hops = min_hops.min(max_hops);
    let max_hops = max_hops.max(min_hops);
    let uniq = uniqueness_factor(uniqueness);
    // Few declared types shrink average fanout; unrestricted types keep the default.
    let type_scale = if type_count == 0 {
        1.0
    } else {
        (type_count as f64).clamp(1.0, 8.0) / 4.0
    };
    let branching = (DEFAULT_BRANCHING * type_scale).max(1.25);

    let mut path_count = 0.0_f64;
    for hops in min_hops..=max_hops {
        path_count += if hops == 0 {
            1.0
        } else {
            branching.powi(hops as i32) * uniq
        };
    }
    path_count = path_count.min(max_paths.max(1) as f64);

    // The vtab emits one row per path position (including the start node at 0).
    let avg_positions = ((min_hops as f64 + max_hops as f64) / 2.0) + 1.0;
    let rows = (path_count * avg_positions)
        .ceil()
        .clamp(1.0, MAX_ESTIMATED_ROWS as f64) as u64;

    // Fixed setup (snapshot lookup + cursor) plus work linear in emitted rows.
    let estimated_cost = 25.0 + (rows as f64) * 0.08;
    ExpandCostEstimate {
        estimated_rows: rows,
        estimated_cost,
    }
}

fn uniqueness_factor(uniqueness: &str) -> f64 {
    match uniqueness {
        "path" => 0.45,
        "trail" => 0.7,
        _ => 1.0,
    }
}

/// Count types in the expand filter string (`"1,2,3"` or empty for all).
pub fn relationship_type_count(type_list: &str) -> u32 {
    let trimmed = type_list.trim();
    if trimmed.is_empty() {
        return 0;
    }
    trimmed
        .split(',')
        .filter(|part| !part.trim().is_empty())
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longer_hop_spans_estimate_more_rows_and_cost() {
        let short = estimate_expand(1, 1, 1, "trail", 100_000);
        let long = estimate_expand(1, 4, 1, "trail", 100_000);
        assert!(long.estimated_rows > short.estimated_rows);
        assert!(long.estimated_cost > short.estimated_cost);
    }

    #[test]
    fn path_uniqueness_estimates_fewer_rows_than_walk() {
        let walk = estimate_expand(1, 3, 1, "walk", 100_000);
        let path = estimate_expand(1, 3, 1, "path", 100_000);
        assert!(path.estimated_rows <= walk.estimated_rows);
    }

    #[test]
    fn max_paths_caps_estimate() {
        let uncapped = estimate_expand(1, 5, 0, "walk", 1_000_000);
        let capped = estimate_expand(1, 5, 0, "walk", 10);
        assert!(capped.estimated_rows < uncapped.estimated_rows);
        assert!(capped.estimated_rows <= 10 * 6); // paths * positions upper band
    }

    #[test]
    fn type_list_count_parses_comma_separated_ids() {
        assert_eq!(relationship_type_count(""), 0);
        assert_eq!(relationship_type_count("1"), 1);
        assert_eq!(relationship_type_count("1,2,3"), 3);
        assert_eq!(relationship_type_count(" 1 , 2 "), 2);
    }

    #[test]
    fn zero_hop_only_is_cheap() {
        let estimate = estimate_expand(0, 0, 0, "trail", 100_000);
        assert_eq!(estimate.estimated_rows, 1);
        assert!(estimate.estimated_cost < 50.0);
    }
}
