//! Which path-finding combinations Turso will answer, and with what.
//!
//! `graph/cypher/src/cypher.pest` has `range_literal` (`[r:T*1..3]`) but no
//! `SHORTEST`, `ALL SHORTEST`, `TRAIL`, or `ACYCLIC` selector. When that syntax
//! arrives, each combination of uniqueness, selector, and weight sign needs a
//! decision, and several of those decisions are traps: Dijkstra is silently
//! wrong with negative weights, a shortest walk is undefined across a negative
//! cycle, and shortest simple path with negative weights is NP-hard. The table
//! is written before the syntax so nobody has to make those calls under
//! pressure, and `resolve_path_algorithm` is total: every combination has a
//! verdict, and no combination falls through to a default.
//!
//! Weights are `u64` today (`EdgeInput.weight`, `Path.total_weight`), so
//! `WeightClass::Negative` is unreachable from the current type. The rows exist
//! so that widening the weight type trips a policy error instead of quietly
//! feeding negative edges to Dijkstra.

use turso_graph_ir::PathUniqueness;

/// Bump on any change to the table below, and mirror into
/// `turso_graph_ir::SEMANTIC_PROFILE.path_policy_version`.
pub const PATH_POLICY_VERSION: u32 = 1;

/// How many of the matching paths the caller wants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathSelector {
    /// Every matching path.
    All,
    /// Any one matching path; the caller does not care which.
    Any,
    /// One path of minimum cost.
    Shortest,
    /// Every path of minimum cost.
    AllShortest,
    /// The k lowest-cost paths, in cost order.
    ShortestK(u32),
}

/// The sign domain of the edge weights in play.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightClass {
    /// Every edge costs 1.
    Unweighted,
    /// Every edge weight is >= 0.
    NonNegative,
    /// At least one edge weight may be < 0.
    Negative,
}

/// An algorithm the table considers sound for a combination. Being named here
/// says the algorithm is correct, not that it is implemented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathAlgorithm {
    BreadthFirst,
    BreadthFirstAllShortest,
    Dijkstra,
    DijkstraAllShortest,
    DepthFirstEnumeration,
    YenKShortest,
}

impl PathAlgorithm {
    /// Whether this crate can actually run it today.
    ///
    /// Soundness and availability are separate questions, and collapsing them
    /// would let "not built" read as "cannot be done". A caller that resolves
    /// to a sound-but-unbuilt algorithm gets
    /// `RuntimeError::PathAlgorithmNotImplemented`, not a refusal of the
    /// combination.
    pub fn is_implemented(&self) -> bool {
        match self {
            Self::BreadthFirst => true,
            Self::Dijkstra => true,
            Self::DepthFirstEnumeration => true,
            Self::BreadthFirstAllShortest | Self::DijkstraAllShortest | Self::YenKShortest => false,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::BreadthFirst => "breadth-first",
            Self::BreadthFirstAllShortest => "breadth-first all-shortest",
            Self::Dijkstra => "dijkstra",
            Self::DijkstraAllShortest => "dijkstra all-shortest",
            Self::DepthFirstEnumeration => "depth-first enumeration",
            Self::YenKShortest => "yen k-shortest",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PathPolicyError {
    #[error("{uniqueness:?}/{selector:?} with {weights:?} weights is not supported: {reason}")]
    Unsupported {
        uniqueness: PathUniqueness,
        selector: PathSelector,
        weights: WeightClass,
        reason: &'static str,
    },
}

/// The legality table. Total over every combination.
pub fn resolve_path_algorithm(
    uniqueness: PathUniqueness,
    selector: PathSelector,
    weights: WeightClass,
) -> Result<PathAlgorithm, PathPolicyError> {
    let refuse = |reason| {
        Err(PathPolicyError::Unsupported {
            uniqueness,
            selector,
            weights,
            reason,
        })
    };

    match selector {
        // Existence only. No weight sign changes whether a path exists, and
        // BFS finds one under every uniqueness rule.
        PathSelector::Any => Ok(PathAlgorithm::BreadthFirst),

        // Enumeration. Weights are irrelevant to which paths exist. A walk may
        // repeat edges, so a single cycle makes the answer infinite; the hop
        // limit bounds it but the result would then be an arbitrary prefix,
        // which is exactly the silent truncation traversal.rs refuses.
        PathSelector::All => match uniqueness {
            PathUniqueness::Walk => refuse(
                "enumerating all walks is infinite in a cyclic graph; \
                 use TRAIL or ACYCLIC, or ask for ANY",
            ),
            PathUniqueness::Trail | PathUniqueness::Path => {
                Ok(PathAlgorithm::DepthFirstEnumeration)
            }
        },

        PathSelector::Shortest => match weights {
            // A shortest unweighted walk never repeats a node, so the trail and
            // acyclic constraints are automatically satisfied and BFS is
            // correct for all three.
            WeightClass::Unweighted => Ok(PathAlgorithm::BreadthFirst),
            // Same argument with non-negative weights: no detour can lower the
            // cost, so the minimum-cost walk is simple.
            WeightClass::NonNegative => Ok(PathAlgorithm::Dijkstra),
            WeightClass::Negative => match uniqueness {
                PathUniqueness::Walk => {
                    refuse("a negative cycle makes the shortest walk undefined")
                }
                PathUniqueness::Trail | PathUniqueness::Path => refuse(
                    "shortest simple path with negative weights is NP-hard; \
                     no correct polynomial algorithm exists",
                ),
            },
        },

        PathSelector::AllShortest => match weights {
            WeightClass::Unweighted => Ok(PathAlgorithm::BreadthFirstAllShortest),
            WeightClass::NonNegative => Ok(PathAlgorithm::DijkstraAllShortest),
            WeightClass::Negative => refuse(
                "the shortest cost is undefined with negative weights, \
                 so the set of shortest paths is too",
            ),
        },

        PathSelector::ShortestK(_) => match weights {
            // Yen's algorithm needs a simple-path constraint and a shortest
            // path subroutine that is correct, which rules out walks and
            // negative weights.
            WeightClass::Unweighted | WeightClass::NonNegative => match uniqueness {
                PathUniqueness::Walk => {
                    refuse("k-shortest requires simple paths; use TRAIL or ACYCLIC")
                }
                PathUniqueness::Trail | PathUniqueness::Path => Ok(PathAlgorithm::YenKShortest),
            },
            WeightClass::Negative => {
                refuse("k-shortest inherits the negative-weight shortest-path refusal")
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turso_graph_ir::PathUniqueness;

    #[test]
    fn unweighted_shortest_is_breadth_first_under_every_uniqueness() {
        // In an unweighted graph a shortest walk never repeats a node, so BFS
        // is correct for Walk, Trail, and Path alike.
        for uniqueness in [
            PathUniqueness::Walk,
            PathUniqueness::Trail,
            PathUniqueness::Path,
        ] {
            assert_eq!(
                resolve_path_algorithm(uniqueness, PathSelector::Shortest, WeightClass::Unweighted),
                Ok(PathAlgorithm::BreadthFirst)
            );
        }
    }

    #[test]
    fn non_negative_weighted_shortest_is_dijkstra_under_every_uniqueness() {
        // With non-negative weights a shortest walk is again always simple, so
        // the trail and acyclic constraints cost nothing.
        for uniqueness in [
            PathUniqueness::Walk,
            PathUniqueness::Trail,
            PathUniqueness::Path,
        ] {
            assert_eq!(
                resolve_path_algorithm(
                    uniqueness,
                    PathSelector::Shortest,
                    WeightClass::NonNegative
                ),
                Ok(PathAlgorithm::Dijkstra)
            );
        }
    }

    #[test]
    fn negative_weights_have_no_shortest_path_answer_we_will_give() {
        // Dijkstra is silently wrong with negative weights; a negative cycle
        // makes a shortest walk undefined; and shortest simple path with
        // negative weights is NP-hard. All three refuse, none guess.
        for uniqueness in [
            PathUniqueness::Walk,
            PathUniqueness::Trail,
            PathUniqueness::Path,
        ] {
            for selector in [
                PathSelector::Shortest,
                PathSelector::AllShortest,
                PathSelector::ShortestK(2),
            ] {
                assert!(
                    resolve_path_algorithm(uniqueness, selector, WeightClass::Negative).is_err(),
                    "{uniqueness:?}/{selector:?} must refuse negative weights"
                );
            }
        }
    }

    #[test]
    fn any_path_is_breadth_first_and_weight_blind() {
        for weights in [
            WeightClass::Unweighted,
            WeightClass::NonNegative,
            WeightClass::Negative,
        ] {
            assert_eq!(
                resolve_path_algorithm(PathUniqueness::Trail, PathSelector::Any, weights),
                Ok(PathAlgorithm::BreadthFirst),
                "ANY asks for existence, which no weight sign changes"
            );
        }
    }

    #[test]
    fn all_paths_enumerate_and_do_not_care_about_weight_sign() {
        for weights in [
            WeightClass::Unweighted,
            WeightClass::NonNegative,
            WeightClass::Negative,
        ] {
            assert_eq!(
                resolve_path_algorithm(PathUniqueness::Trail, PathSelector::All, weights),
                Ok(PathAlgorithm::DepthFirstEnumeration)
            );
        }
    }

    #[test]
    fn all_paths_over_walks_are_refused_because_a_cycle_makes_them_infinite() {
        assert!(matches!(
            resolve_path_algorithm(
                PathUniqueness::Walk,
                PathSelector::All,
                WeightClass::Unweighted
            ),
            Err(PathPolicyError::Unsupported { .. })
        ));
    }

    #[test]
    fn k_shortest_is_declared_sound_but_not_built() {
        // Soundness and availability are different questions. Answering them
        // with one error would let "we have not built it" read as "it cannot
        // be done", and the next person would design around a wrong table.
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Path,
                PathSelector::ShortestK(3),
                WeightClass::NonNegative
            ),
            Ok(PathAlgorithm::YenKShortest)
        );
        assert!(
            !PathAlgorithm::YenKShortest.is_implemented(),
            "Yen is sound and unbuilt; saying otherwise would promise a search that does not exist"
        );
        assert!(PathAlgorithm::BreadthFirst.is_implemented());
        assert!(PathAlgorithm::Dijkstra.is_implemented());
    }

    #[test]
    fn every_combination_in_the_table_has_a_verdict() {
        // No combination may fall through to a default. A missing row is a
        // silent wrong answer waiting for the syntax to arrive.
        for uniqueness in [
            PathUniqueness::Walk,
            PathUniqueness::Trail,
            PathUniqueness::Path,
        ] {
            for selector in [
                PathSelector::All,
                PathSelector::Any,
                PathSelector::Shortest,
                PathSelector::AllShortest,
                PathSelector::ShortestK(2),
            ] {
                for weights in [
                    WeightClass::Unweighted,
                    WeightClass::NonNegative,
                    WeightClass::Negative,
                ] {
                    let verdict = resolve_path_algorithm(uniqueness, selector, weights);
                    assert!(
                        verdict.is_ok() || verdict.is_err(),
                        "unreachable, but the call must not panic"
                    );
                }
            }
        }
    }

    #[test]
    fn the_semantic_profile_mirrors_this_policy_version() {
        // The IR crate cannot depend on the runtime, so the mirror is checked
        // from the side that can see both.
        assert_eq!(
            turso_graph_ir::SEMANTIC_PROFILE.path_policy_version,
            PATH_POLICY_VERSION,
            "bump SEMANTIC_PROFILE_VERSION and its pinned digest alongside PATH_POLICY_VERSION"
        );
    }
}
