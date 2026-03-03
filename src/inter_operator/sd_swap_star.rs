//! SdSwapStar operator implementation.
//!
//! Split Delivery SwapStar - allows splitting loads during exchange with star cache optimization.

use super::base_cache::{BaseCache, InterRouteCache};
use super::base_star::StarCaches;
use super::{calc_delta, InterOperator};
use crate::cache::CacheMap;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Move data for the SdSwapStar operator.
#[derive(Clone, Default)]
struct SdSwapStarMove {
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

/// SdSwapStar operator.
///
/// Split Delivery version of SwapStar. When exchanging nodes between routes,
/// allows splitting the load to maintain capacity constraints.
#[derive(Debug, Clone, Default)]
pub struct SdSwapStar;

impl SdSwapStar {
    /// Applies a SD swap star move.
    fn do_sd_swap_star(mv: &SdSwapStarMove, solution: &mut AlkaidSolution, context: &mut RouteContext) {
        let predecessor_y = solution.predecessor(mv.node_y);
        let successor_y = solution.successor(mv.node_y);

        // Update route X: adjust load and insert node_y at its best position (predecessor_x, successor_x)
        solution.set_successor(0, context.head(mv.route_x));
        solution.set_load(mv.node_x, mv.split_load);
        solution.link(mv.predecessor_x, mv.node_y);
        solution.link(mv.node_y, mv.successor_x);
        context.set_head(mv.route_x, solution.successor(0));

        // Update route Y: remove old node_y, insert new node at best position (predecessor_y, successor_y)
        solution.set_successor(0, context.head(mv.route_y));
        solution.link(predecessor_y, successor_y);
        let customer_x = solution.customer(mv.node_x);
        let load_y = solution.load(mv.node_y);
        solution.insert(customer_x, load_y, mv.predecessor_y, mv.successor_y);
        context.set_head(mv.route_y, solution.successor(0));
    }

    /// Evaluates a single node pair for SD swap star.
    #[allow(clippy::too_many_arguments)]
    fn sd_swap_star_inner_single(
        instance: &Instance,
        solution: &AlkaidSolution,
        swapped: bool,
        route_x: Node,
        route_y: Node,
        node_x: Node,
        node_y: Node,
        split_load: i32,
        cache: &mut BaseCache<SdSwapStarMove>,
        star_caches: &StarCaches,
        random: &mut Random,
    ) {
        let insertion_x = star_caches.get(route_y, solution.customer(node_x));
        let insertion_y = star_caches.get(route_x, solution.customer(node_y));
        
        let mut predecessor_y = solution.predecessor(node_y);
        let mut successor_y = solution.successor(node_y);
        
        let delta = -calc_delta(instance, solution, node_y, predecessor_y, successor_y);
        let mut delta_x = calc_delta(instance, solution, node_x, predecessor_y, successor_y);
        
        // Try to find better insertion for node_x using star cache
        if let Some(best_insertion_x) = insertion_x.find_best_without_node(node_y) {
            if best_insertion_x.delta.value < delta_x {
                delta_x = best_insertion_x.delta.value;
                predecessor_y = best_insertion_x.predecessor;
                successor_y = best_insertion_x.successor;
            }
        }
        
        // Find best insertion for node_y using star cache
        let best_insertion_y = insertion_y.find_best();
        let total_delta = delta + delta_x + best_insertion_y.delta.value;
            
        if cache.delta.update(total_delta, random) {
            cache.mv = SdSwapStarMove {
                swapped,
                route_x,
                route_y,
                node_x,
                predecessor_y,
                successor_y,
                node_y,
                predecessor_x: best_insertion_y.predecessor,
                successor_x: best_insertion_y.successor,
                split_load,
            };
        }
    }

    /// Inner function to evaluate all possible SD swaps between two routes.
    #[allow(clippy::too_many_arguments)]
    fn sd_swap_star_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route_x: Node,
        route_y: Node,
        cache: &mut BaseCache<SdSwapStarMove>,
        star_caches: &StarCaches,
        random: &mut Random,
    ) {
        let mut node_x = context.head(route_x);
        while node_x != 0 {
            let load_x = solution.load(node_x);

            let mut node_y = context.head(route_y);
            while node_y != 0 {
                let load_y = solution.load(node_y);

                if load_x > load_y {
                    Self::sd_swap_star_inner_single(
                        instance, solution, false, route_x, route_y, node_x, node_y,
                        load_x - load_y, cache, star_caches, random,
                    );
                } else if load_y > load_x {
                    Self::sd_swap_star_inner_single(
                        instance, solution, true, route_y, route_x, node_y, node_x,
                        load_y - load_x, cache, star_caches, random,
                    );
                }

                node_y = solution.successor(node_y);
            }

            node_x = solution.successor(node_x);
        }
    }
}

impl InterOperator for SdSwapStar {
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node> {
        let (caches, star_caches) = cache_map
            .get2_mut::<InterRouteCache<SdSwapStarMove>, StarCaches>(solution, context);
        let mut best_move = SdSwapStarMove::default();
        let mut best_delta = Delta::default();

        for route_x in 0..context.num_routes() {
            for route_y in (route_x + 1)..context.num_routes() {
                let cache = caches.get(route_x, route_y);
                if !cache.try_reuse() {
                    star_caches.preprocess(instance, solution, context, route_x, random);
                    star_caches.preprocess(instance, solution, context, route_y, random);
                    Self::sd_swap_star_inner(
                        instance, solution, context, route_x, route_y, cache, star_caches, random,
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
            Self::do_sd_swap_star(&best_move, solution, context);
            vec![best_move.route_x, best_move.route_y]
        } else {
            vec![]
        }
    }
}
