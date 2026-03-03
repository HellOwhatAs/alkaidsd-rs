//! SwapStar operator implementation.
//!
//! Exchanges single nodes between routes using star cache optimization.

use super::base_cache::{BaseCache, InterRouteCache};
use super::base_star::StarCaches;
use super::{calc_delta, InterOperator};
use crate::cache::CacheMap;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Move data for the SwapStar operator.
#[derive(Clone, Default)]
struct SwapStarMove {
    route_x: Node,
    route_y: Node,
    node_x: Node,
    predecessor_x: Node,
    successor_x: Node,
    node_y: Node,
    predecessor_y: Node,
    successor_y: Node,
}

/// SwapStar operator.
///
/// Exchanges single nodes between two routes, finding optimal insertion
/// positions for each using star cache optimization.
#[derive(Debug, Clone, Default)]
pub struct SwapStar;

impl SwapStar {
    /// Applies a swap star move.
    fn do_swap_star(mv: &SwapStarMove, solution: &mut AlkaidSolution, context: &mut RouteContext) {
        let predecessor_x = solution.predecessor(mv.node_x);
        let successor_x = solution.successor(mv.node_x);
        let predecessor_y = solution.predecessor(mv.node_y);
        let successor_y = solution.successor(mv.node_y);

        // Update route X: remove node_x, insert node_y at predecessor_x/successor_x
        // (predecessor_x/successor_x hold the insertion position for node_y in route_x)
        solution.set_successor(0, context.head(mv.route_x));
        solution.link(predecessor_x, successor_x);
        solution.link(mv.predecessor_x, mv.node_y);
        solution.link(mv.node_y, mv.successor_x);
        context.set_head(mv.route_x, solution.successor(0));

        // Update route Y: remove node_y, insert node_x at predecessor_y/successor_y
        // (predecessor_y/successor_y hold the insertion position for node_x in route_y)
        solution.set_successor(0, context.head(mv.route_y));
        solution.link(predecessor_y, successor_y);
        solution.link(mv.predecessor_y, mv.node_x);
        solution.link(mv.node_x, mv.successor_y);
        context.set_head(mv.route_y, solution.successor(0));
    }

    /// Inner function to evaluate all possible swaps between two routes.
    #[allow(clippy::too_many_arguments)]
    fn swap_star_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route_x: Node,
        route_y: Node,
        cache: &mut BaseCache<SwapStarMove>,
        star_caches: &StarCaches,
        random: &mut Random,
    ) {
        let mut node_x = context.head(route_x);
        while node_x != 0 {
            let insertion_x = star_caches.get(route_y, solution.customer(node_x));
            let load_x = solution.load(node_x);
            let load_y_lower = -instance.capacity + context.load(route_y) + load_x;
            let load_y_upper = instance.capacity - context.load(route_x) + load_x;

            let mut node_y = context.head(route_y);
            while node_y != 0 {
                let load_y = solution.load(node_y);

                if load_y >= load_y_lower && load_y <= load_y_upper {
                    let insertion_y = star_caches.get(route_x, solution.customer(node_y));

                    let predecessor_x_orig = solution.predecessor(node_x);
                    let successor_x_orig = solution.successor(node_x);
                    let predecessor_y_orig = solution.predecessor(node_y);
                    let successor_y_orig = solution.successor(node_y);

                    // Calculate removal costs
                    let delta = -calc_delta(instance, solution, node_x, predecessor_x_orig, successor_x_orig)
                        - calc_delta(instance, solution, node_y, predecessor_y_orig, successor_y_orig);

                    // Default: insert at each other's original positions
                    let mut delta_x = calc_delta(instance, solution, node_x, predecessor_y_orig, successor_y_orig);
                    let mut predecessor_y = predecessor_y_orig;
                    let mut successor_y = successor_y_orig;

                    // Check if star cache has a better position for node_x (excluding node_y)
                    if let Some(best_insertion_x) = insertion_x.find_best_without_node(node_y) {
                        if best_insertion_x.delta.value < delta_x {
                            delta_x = best_insertion_x.delta.value;
                            predecessor_y = best_insertion_x.predecessor;
                            successor_y = best_insertion_x.successor;
                        }
                    }

                    let mut delta_y = calc_delta(instance, solution, node_y, predecessor_x_orig, successor_x_orig);
                    let mut predecessor_x = predecessor_x_orig;
                    let mut successor_x = successor_x_orig;

                    // Check if star cache has a better position for node_y (excluding node_x)
                    if let Some(best_insertion_y) = insertion_y.find_best_without_node(node_x) {
                        if best_insertion_y.delta.value < delta_y {
                            delta_y = best_insertion_y.delta.value;
                            predecessor_x = best_insertion_y.predecessor;
                            successor_x = best_insertion_y.successor;
                        }
                    }

                    let total_delta = delta + delta_x + delta_y;

                    if cache.delta.update(total_delta, random) {
                        cache.mv = SwapStarMove {
                            route_x,
                            route_y,
                            node_x,
                            predecessor_x,
                            successor_x,
                            node_y,
                            predecessor_y,
                            successor_y,
                        };
                    }
                }

                node_y = solution.successor(node_y);
            }

            node_x = solution.successor(node_x);
        }
    }
}

impl InterOperator for SwapStar {
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node> {
        let (caches, star_caches) = cache_map
            .get2_mut::<InterRouteCache<SwapStarMove>, StarCaches>(solution, context);
        let mut best_move = SwapStarMove::default();
        let mut best_delta = Delta::default();

        for route_x in 0..context.num_routes() {
            for route_y in (route_x + 1)..context.num_routes() {
                let cache = caches.get(route_x, route_y);
                if !cache.try_reuse() {
                    star_caches.preprocess(instance, solution, context, route_x, random);
                    star_caches.preprocess(instance, solution, context, route_y, random);
                    Self::swap_star_inner(
                        instance, solution, context, route_x, route_y, cache, star_caches, random,
                    );
                } else {
                    cache.mv.route_x = route_x;
                    cache.mv.route_y = route_y;
                }
                if best_delta.update_from(&cache.delta, random) {
                    best_move = cache.mv.clone();
                }
            }
        }

        if best_delta.value < 0 {
            Self::do_swap_star(&best_move, solution, context);
            vec![best_move.route_x, best_move.route_y]
        } else {
            vec![]
        }
    }
}
