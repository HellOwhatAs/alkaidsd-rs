//! SdSwapTwoOne operator implementation.
//!
//! Split Delivery Swap(2,1) - exchanges pairs with singles, with load splitting.

use super::base_cache::{BaseCache, InterRouteCache};
use super::InterOperator;
use crate::cache::CacheMap;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Move data for the SdSwapTwoOne operator.
#[derive(Clone, Default)]
struct SdSwapTwoOneMove {
    move_type: i32,  // 0 or 1
    route_ij: Node,
    route_k: Node,
    predecessor_ij: Node,
    successor_ij: Node,
    node_i: Node,
    node_j: Node,
    node_k: Node,
    split_load: i32,
    direction_ij: bool,
    direction_ijk: bool,
}

/// SdSwapTwoOne operator.
///
/// Split Delivery version of Swap(2,1). Exchanges a pair of consecutive nodes
/// with a single node while allowing load splitting.
#[derive(Debug, Clone, Default)]
pub struct SdSwapTwoOne;

impl SdSwapTwoOne {
    /// Applies a SD swap(2,1) move.
    fn do_sd_swap_two_one(mv: &SdSwapTwoOneMove, solution: &mut AlkaidSolution, context: &mut RouteContext) {
        let predecessor_k = solution.predecessor(mv.node_k);
        let successor_k = solution.successor(mv.node_k);

        // Remove from both routes
        solution.set_successor(0, context.head(mv.route_ij));
        solution.link(mv.predecessor_ij, mv.successor_ij);
        context.set_head(mv.route_ij, solution.successor(0));

        solution.set_successor(0, context.head(mv.route_k));
        solution.link(predecessor_k, successor_k);
        context.set_head(mv.route_k, solution.successor(0));

        if mv.move_type == 0 {
            // Type 0: split load from node_j
            let customer_j = solution.customer(mv.node_j);
            let new_node_j = solution.new_node(customer_j, mv.split_load);
            solution.set_load(mv.node_j, solution.load(mv.node_j) - mv.split_load);

            // Insert into route_ij
            solution.set_successor(0, context.head(mv.route_ij));
            if mv.direction_ijk {
                solution.link(mv.predecessor_ij, new_node_j);
                solution.link(new_node_j, mv.node_k);
                solution.link(mv.node_k, mv.successor_ij);
            } else {
                solution.link(mv.predecessor_ij, mv.node_k);
                solution.link(mv.node_k, new_node_j);
                solution.link(new_node_j, mv.successor_ij);
            }
            context.set_head(mv.route_ij, solution.successor(0));

            // Insert into route_k
            solution.set_successor(0, context.head(mv.route_k));
            if mv.direction_ij {
                solution.link(predecessor_k, mv.node_i);
                solution.link(mv.node_i, mv.node_j);
                solution.link(mv.node_j, successor_k);
            } else {
                solution.link(predecessor_k, mv.node_j);
                solution.link(mv.node_j, mv.node_i);
                solution.link(mv.node_i, successor_k);
            }
            context.set_head(mv.route_k, solution.successor(0));
        } else {
            // Type 1: split load from node_k
            let customer_k = solution.customer(mv.node_k);
            let new_node_k = solution.new_node(customer_k, mv.split_load);
            solution.set_load(mv.node_k, solution.load(mv.node_k) - mv.split_load);

            // Insert into route_ij
            solution.set_successor(0, context.head(mv.route_ij));
            solution.link(mv.predecessor_ij, mv.node_k);
            solution.link(mv.node_k, mv.successor_ij);
            context.set_head(mv.route_ij, solution.successor(0));

            // Insert into route_k
            solution.set_successor(0, context.head(mv.route_k));
            let (before_ij, after_ij) = if mv.direction_ij {
                (mv.node_i, mv.node_j)
            } else {
                (mv.node_j, mv.node_i)
            };

            solution.link(predecessor_k, before_ij);
            solution.link(before_ij, after_ij);
            solution.link(after_ij, successor_k);

            if mv.direction_ijk {
                solution.link(after_ij, new_node_k);
                solution.link(new_node_k, successor_k);
            } else {
                solution.link(predecessor_k, new_node_k);
                solution.link(new_node_k, before_ij);
            }
            context.set_head(mv.route_k, solution.successor(0));
        }
    }

    /// Evaluates Type 0 moves for a given (node_i, node_j, node_k) triple.
    #[allow(clippy::too_many_arguments)]
    fn sd_swap_two_one0(
        instance: &Instance,
        solution: &AlkaidSolution,
        route_ij: Node,
        route_k: Node,
        node_i: Node,
        node_j: Node,
        node_k: Node,
        predecessor_ij: Node,
        successor_ij: Node,
        split_load: i32,
        base_delta: i32,
        cache: &mut BaseCache<SdSwapTwoOneMove>,
        random: &mut Random,
    ) {
        let predecessor_k = solution.predecessor(node_k);
        let successor_k = solution.successor(node_k);

        let delta_ij = instance.distance(solution.customer(predecessor_k), solution.customer(node_i))
            + instance.distance(solution.customer(node_j), solution.customer(successor_k));
        let delta_ji = instance.distance(solution.customer(predecessor_k), solution.customer(node_j))
            + instance.distance(solution.customer(node_i), solution.customer(successor_k));

        let delta_jk = instance.distance(solution.customer(predecessor_ij), solution.customer(node_j))
            + instance.distance(solution.customer(node_k), solution.customer(successor_ij));
        let delta_kj = instance.distance(solution.customer(predecessor_ij), solution.customer(node_k))
            + instance.distance(solution.customer(node_j), solution.customer(successor_ij));

        let mut best_delta_ij = delta_ij;
        let mut direction_ij = true;
        if delta_ij > delta_ji {
            best_delta_ij = delta_ji;
            direction_ij = false;
        }

        let mut best_delta_jk = delta_jk;
        let mut direction_jk = true;
        if delta_jk > delta_kj {
            best_delta_jk = delta_kj;
            direction_jk = false;
        }

        let delta = base_delta
            + instance.distance(solution.customer(node_j), solution.customer(node_k))
            + best_delta_ij
            + best_delta_jk;

        if cache.delta.update(delta, random) {
            cache.mv = SdSwapTwoOneMove {
                move_type: 0,
                route_ij,
                route_k,
                predecessor_ij,
                successor_ij,
                node_i,
                node_j,
                node_k,
                split_load,
                direction_ij,
                direction_ijk: direction_jk,
            };
        }
    }

    /// Evaluates Type 1 moves for a given (node_i, node_j, node_k) triple.
    #[allow(clippy::too_many_arguments)]
    fn sd_swap_two_one1(
        instance: &Instance,
        solution: &AlkaidSolution,
        route_ij: Node,
        route_k: Node,
        node_i: Node,
        node_j: Node,
        node_k: Node,
        predecessor_ij: Node,
        successor_ij: Node,
        split_load: i32,
        base_delta: i32,
        cache: &mut BaseCache<SdSwapTwoOneMove>,
        random: &mut Random,
    ) {
        let predecessor_k = solution.predecessor(node_k);
        let successor_k = solution.successor(node_k);

        let base_delta = base_delta
            + instance.distance(solution.customer(predecessor_ij), solution.customer(node_k))
            + instance.distance(solution.customer(node_k), solution.customer(successor_ij));

        for direction_ij in [true, false] {
            let (before_ij, after_ij) = if direction_ij {
                (node_i, node_j)
            } else {
                (node_j, node_i)
            };

            for direction_ijk in [true, false] {
                let delta_ijk = if direction_ijk {
                    instance.distance(solution.customer(predecessor_k), solution.customer(before_ij))
                        + instance.distance(solution.customer(after_ij), solution.customer(node_k))
                        + instance.distance(solution.customer(node_k), solution.customer(successor_k))
                } else {
                    instance.distance(solution.customer(predecessor_k), solution.customer(node_k))
                        + instance.distance(solution.customer(node_k), solution.customer(before_ij))
                        + instance.distance(solution.customer(after_ij), solution.customer(successor_k))
                };

                let delta = base_delta + delta_ijk;
                if cache.delta.update(delta, random) {
                    cache.mv = SdSwapTwoOneMove {
                        move_type: 1,
                        route_ij,
                        route_k,
                        predecessor_ij,
                        successor_ij,
                        node_i,
                        node_j,
                        node_k,
                        split_load,
                        direction_ij,
                        direction_ijk,
                    };
                }
            }
        }
    }

    /// Evaluates all moves for a single route pair.
    fn sd_swap_two_one_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route_ij: Node,
        route_k: Node,
        cache: &mut BaseCache<SdSwapTwoOneMove>,
        random: &mut Random,
    ) {
        let mut node_i = context.head(route_ij);
        let mut node_j = solution.successor(node_i);

        while node_j != 0 {
            let load_i = solution.load(node_i);
            let load_j = solution.load(node_j);

            let mut node_k = context.head(route_k);
            while node_k != 0 {
                let load_k = solution.load(node_k);

                let predecessor_ij = solution.predecessor(node_i);
                let successor_ij = solution.successor(node_j);

                let base_delta = -instance.distance(solution.customer(predecessor_ij), solution.customer(node_i))
                    - instance.distance(solution.customer(node_j), solution.customer(successor_ij))
                    - instance.distance(solution.customer(solution.predecessor(node_k)), solution.customer(node_k))
                    - instance.distance(solution.customer(node_k), solution.customer(solution.successor(node_k)));

                if load_i + load_j > load_k {
                    if load_i < load_k {
                        Self::sd_swap_two_one0(
                            instance, solution, route_ij, route_k, node_i, node_j, node_k,
                            predecessor_ij, successor_ij, load_i + load_j - load_k, base_delta,
                            cache, random,
                        );
                    }
                    if load_j < load_k {
                        Self::sd_swap_two_one0(
                            instance, solution, route_ij, route_k, node_j, node_i, node_k,
                            predecessor_ij, successor_ij, load_i + load_j - load_k, base_delta,
                            cache, random,
                        );
                    }
                } else if load_k > load_i + load_j {
                    Self::sd_swap_two_one1(
                        instance, solution, route_ij, route_k, node_i, node_j, node_k,
                        predecessor_ij, successor_ij, load_k - load_i - load_j, base_delta,
                        cache, random,
                    );
                }

                node_k = solution.successor(node_k);
            }

            node_i = node_j;
            node_j = solution.successor(node_j);
        }
    }
}

impl InterOperator for SdSwapTwoOne {
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node> {
        let caches: &mut InterRouteCache<SdSwapTwoOneMove> = cache_map.get(solution, context);
        let mut best_move = SdSwapTwoOneMove::default();
        let mut best_delta = Delta::default();

        for route_ij in 0..context.num_routes() {
            for route_k in 0..context.num_routes() {
                if route_ij == route_k {
                    continue;
                }

                let cache = caches.get(route_ij, route_k);
                if !cache.try_reuse() {
                    Self::sd_swap_two_one_inner(
                        instance, solution, context, route_ij, route_k, cache, random,
                    );
                } else {
                    cache.mv.route_ij = route_ij;
                    cache.mv.route_k = route_k;
                }
                if best_delta.update_from(&cache.delta, random) {
                    best_move = cache.mv.clone();
                }
            }
        }

        if best_delta.value < 0 {
            Self::do_sd_swap_two_one(&best_move, solution, context);
            vec![best_move.route_ij, best_move.route_k]
        } else {
            vec![]
        }
    }
}
