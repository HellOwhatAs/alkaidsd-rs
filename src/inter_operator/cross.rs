//! Cross operator implementation.
//!
//! Exchanges tail segments between two routes.

use super::base_cache::{BaseCache, InterRouteCache};
use super::InterOperator;
use crate::cache::CacheMap;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Move data for the Cross operator.
#[derive(Clone, Default)]
struct CrossMove {
    reversed: bool,
    route_x: Node,
    route_y: Node,
    left_x: Node,
    left_y: Node,
}

/// Cross operator.
///
/// Exchanges the tail portions of two routes starting from given positions.
/// Can also perform reversed exchanges (2-opt style moves between routes).
#[derive(Debug, Clone, Default)]
pub struct Cross;

impl Cross {
    /// Applies a cross move.
    fn do_cross(mv: &CrossMove, solution: &mut AlkaidSolution, context: &mut RouteContext) {
        let right_x = if mv.left_x != 0 {
            solution.successor(mv.left_x)
        } else {
            context.head(mv.route_x)
        };
        let right_y = if mv.left_y != 0 {
            solution.successor(mv.left_y)
        } else {
            context.head(mv.route_y)
        };

        if !mv.reversed {
            // Simple cross: swap tails
            solution.link(mv.left_x, right_y);
            solution.link(mv.left_y, right_x);

            if mv.left_x == 0 {
                context.set_head(mv.route_x, right_y);
            }
            if mv.left_y == 0 {
                context.set_head(mv.route_y, right_x);
            }
        } else {
            // Reversed cross
            let head_y = context.head(mv.route_y);

            if right_x != 0 {
                let tail_x = context.tail(mv.route_x);
                solution.reversed_link(right_x, tail_x, 0, right_y);
                context.set_head(mv.route_y, tail_x);
            } else {
                solution.link(0, right_y);
                context.set_head(mv.route_y, right_y);
            }

            solution.set_successor(0, context.head(mv.route_x));
            if mv.left_y != 0 {
                solution.reversed_link(head_y, mv.left_y, mv.left_x, 0);
            } else {
                solution.link(mv.left_x, 0);
            }
            context.set_head(mv.route_x, solution.successor(0));
        }
    }

    /// Inner evaluation loop for cross moves.
    fn cross_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route_x: Node,
        route_y: Node,
        cache: &mut BaseCache<CrossMove>,
        random: &mut Random,
    ) {
        let mut left_x = 0;

        loop {
            let successor_x = if left_x != 0 {
                solution.successor(left_x)
            } else {
                context.head(route_x)
            };

            let mut left_y = 0;

            loop {
                let mut predecessor_y = left_y;
                let mut successor_y = if left_y != 0 {
                    solution.successor(left_y)
                } else {
                    context.head(route_y)
                };

                let predecessor_load_x = context.pre_load(left_x);
                let successor_load_x = context.load(route_x) - predecessor_load_x;
                let mut predecessor_load_y = context.pre_load(left_y);
                let mut successor_load_y = context.load(route_y) - predecessor_load_y;

                let base = -instance.distance(solution.customer(left_x), solution.customer(successor_x))
                    - instance.distance(solution.customer(left_y), solution.customer(successor_y));

                for reversed in [false, true] {
                    if predecessor_load_x + successor_load_y <= instance.capacity
                        && successor_load_x + predecessor_load_y <= instance.capacity
                    {
                        let delta = base
                            + instance.distance(solution.customer(left_x), solution.customer(successor_y))
                            + instance.distance(solution.customer(successor_x), solution.customer(predecessor_y));

                        if cache.delta.update(delta, random) {
                            cache.mv = CrossMove {
                                reversed,
                                route_x,
                                route_y,
                                left_x,
                                left_y,
                            };
                        }
                    }

                    // Swap for reversed case
                    std::mem::swap(&mut predecessor_y, &mut successor_y);
                    std::mem::swap(&mut predecessor_load_y, &mut successor_load_y);
                }

                left_y = successor_y;
                if left_y == 0 {
                    break;
                }
            }

            left_x = successor_x;
            if left_x == 0 {
                break;
            }
        }
    }
}

impl InterOperator for Cross {
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node> {
        let caches: &mut InterRouteCache<CrossMove> = cache_map.get(solution, context);

        let mut best_move = CrossMove::default();
        let mut best_delta = Delta::default();

        for route_x in 0..context.num_routes() {
            for route_y in (route_x + 1)..context.num_routes() {
                let cache = caches.get(route_x, route_y);
                if !cache.try_reuse() {
                    Self::cross_inner(instance, solution, context, route_x, route_y, cache, random);
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
            Self::do_cross(&best_move, solution, context);
            vec![best_move.route_x, best_move.route_y]
        } else {
            vec![]
        }
    }
}
