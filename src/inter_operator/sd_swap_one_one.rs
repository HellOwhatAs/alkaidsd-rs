//! SdSwapOneOne operator implementation.
//!
//! Split Delivery Swap(1,1) - exchanges single nodes with load splitting.

use super::base_cache::{BaseCache, InterRouteCache};
use super::{calc_delta, InterOperator};
use crate::cache::CacheMap;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Move data for the SdSwapOneOne operator.
#[derive(Clone, Default)]
struct SdSwapOneOneMove {
    /// Whether the routes were swapped during evaluation (for route ordering)
    swapped: bool,
    route_x: Node,
    route_y: Node,
    node_x: Node,
    predecessor_x: Node,
    successor_x: Node,
    node_y: Node,
    predecessor_y: Node,
    successor_y: Node,
    split_load: i32,
}

/// SdSwapOneOne operator.
///
/// Split Delivery version of Swap(1,1). Exchanges single nodes between routes
/// while allowing load splitting to handle capacity differences.
#[derive(Debug, Clone, Default)]
pub struct SdSwapOneOne;

impl SdSwapOneOne {
    /// Applies a SD swap(1,1) move.
    fn do_sd_swap_one_one(mv: &SdSwapOneOneMove, solution: &mut AlkaidSolution, context: &mut RouteContext) {
        let predecessor_y = solution.predecessor(mv.node_y);
        let successor_y = solution.successor(mv.node_y);

        // Update route X: insert node_y at its best position (predecessor_x, successor_x)
        solution.set_successor(0, context.head(mv.route_x));
        solution.set_load(mv.node_x, mv.split_load);
        solution.link(mv.predecessor_x, mv.node_y);
        solution.link(mv.node_y, mv.successor_x);
        context.set_head(mv.route_x, solution.successor(0));

        // Update route Y: remove node_y, insert new node at its best position (predecessor_y, successor_y)
        solution.set_successor(0, context.head(mv.route_y));
        solution.link(predecessor_y, successor_y);
        let customer_x = solution.customer(mv.node_x);
        let load_y = solution.load(mv.node_y);
        solution.insert(customer_x, load_y, mv.predecessor_y, mv.successor_y);
        context.set_head(mv.route_y, solution.successor(0));
    }

    /// Evaluates a single node pair for the SD swap(1,1) move.
    #[allow(clippy::too_many_arguments)]
    fn sd_swap_one_one_inner_eval(
        instance: &Instance,
        solution: &AlkaidSolution,
        swapped: bool,
        route_x: Node,
        route_y: Node,
        node_x: Node,
        node_y: Node,
        split_load: i32,
        cache: &mut BaseCache<SdSwapOneOneMove>,
        random: &mut Random,
    ) {
        let predecessor_x = solution.predecessor(node_x);
        let successor_x = solution.successor(node_x);
        let predecessor_y = solution.predecessor(node_y);
        let successor_y = solution.successor(node_y);

        let delta = -calc_delta(instance, solution, node_y, predecessor_y, successor_y);
        let delta_x = calc_delta(instance, solution, node_x, predecessor_y, successor_y);
        let before = calc_delta(instance, solution, node_y, predecessor_x, node_x);
        let after = calc_delta(instance, solution, node_y, node_x, successor_x);

        let (predecessor, successor, delta_y) = if before <= after {
            (predecessor_x, node_x, before)
        } else {
            (node_x, successor_x, after)
        };

        let total_delta = delta + delta_x + delta_y;
        if cache.delta.update(total_delta, random) {
            cache.mv = SdSwapOneOneMove {
                swapped,
                route_x,
                route_y,
                node_x,
                predecessor_y,
                successor_y,
                node_y,
                predecessor_x: predecessor,
                successor_x: successor,
                split_load,
            };
        }
    }

    /// Evaluates all moves for a single route pair.
    fn sd_swap_one_one_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route_x: Node,
        route_y: Node,
        cache: &mut BaseCache<SdSwapOneOneMove>,
        random: &mut Random,
    ) {
        let mut node_x = context.head(route_x);
        while node_x != 0 {
            let load_x = solution.load(node_x);
            let mut node_y = context.head(route_y);
            while node_y != 0 {
                let load_y = solution.load(node_y);
                if load_x > load_y {
                    Self::sd_swap_one_one_inner_eval(
                        instance, solution, false, route_x, route_y,
                        node_x, node_y, load_x - load_y, cache, random,
                    );
                } else if load_y > load_x {
                    Self::sd_swap_one_one_inner_eval(
                        instance, solution, true, route_y, route_x,
                        node_y, node_x, load_y - load_x, cache, random,
                    );
                }
                node_y = solution.successor(node_y);
            }
            node_x = solution.successor(node_x);
        }
    }
}

impl InterOperator for SdSwapOneOne {
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node> {
        let caches: &mut InterRouteCache<SdSwapOneOneMove> = cache_map.get(solution, context);
        let mut best_move = SdSwapOneOneMove::default();
        let mut best_delta = Delta::default();

        for route_x in 0..context.num_routes() {
            for route_y in (route_x + 1)..context.num_routes() {
                let cache = caches.get(route_x, route_y);
                if !cache.try_reuse() {
                    Self::sd_swap_one_one_inner(
                        instance, solution, context, route_x, route_y, cache, random,
                    );
                } else {
                    if !cache.mv.swapped {
                        cache.mv.route_x = route_x;
                        cache.mv.route_y = route_y;
                    } else {
                        cache.mv.route_x = route_y;
                        cache.mv.route_y = route_x;
                    }
                }
                if best_delta.update_from(&cache.delta, random) {
                    best_move = cache.mv.clone();
                }
            }
        }

        if best_delta.value < 0 {
            Self::do_sd_swap_one_one(&best_move, solution, context);
            vec![best_move.route_x, best_move.route_y]
        } else {
            vec![]
        }
    }
}
