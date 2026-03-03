//! Swap operator implementation.
//!
//! Exchanges segments of nodes between two routes.

use super::base_cache::{BaseCache, InterRouteCache};
use super::InterOperator;
use crate::cache::CacheMap;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use std::marker::PhantomData;

/// Move data for the Swap operator.
#[derive(Clone, Default)]
struct SwapMove<const NUM_X: usize, const NUM_Y: usize> {
    route_x: Node,
    route_y: Node,
    direction_x: i32,
    direction_y: i32,
    left_x: Node,
    left_y: Node,
    right_x: Node,
    right_y: Node,
}

/// Generic Swap operator.
///
/// Exchanges NUM_X consecutive nodes from route X with NUM_Y consecutive
/// nodes from route Y. Special case: NUM_Y=0 means shift (no exchange).
///
/// # Type Parameters
///
/// * `NUM_X` - Number of nodes to move from route X
/// * `NUM_Y` - Number of nodes to move from route Y (0 = shift only)
#[derive(Debug, Clone)]
pub struct Swap<const NUM_X: usize, const NUM_Y: usize>(PhantomData<()>);

impl<const NUM_X: usize, const NUM_Y: usize> Default for Swap<NUM_X, NUM_Y> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<const NUM_X: usize, const NUM_Y: usize> Swap<NUM_X, NUM_Y> {
    /// Helper to insert a segment, optionally reversed.
    #[allow(clippy::too_many_arguments)]
    fn segment_insertion(
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        left: Node,
        right: Node,
        predecessor: Node,
        successor: Node,
        route_index: Node,
        direction: i32,
    ) {
        if direction != 0 {
            solution.reversed_link(left, right, predecessor, successor);
        } else {
            solution.link(predecessor, left);
            solution.link(right, successor);
        }
        if predecessor == 0 {
            context.set_head(route_index, if direction != 0 { right } else { left });
        }
    }

    /// Applies a swap move.
    fn do_swap(
        mv: &SwapMove<NUM_X, NUM_Y>,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
    ) {
        if mv.direction_y == -1 {
            // Shift operation (NUM_Y == 0)
            let predecessor = solution.predecessor(mv.left_x);
            let successor = solution.successor(mv.right_x);
            solution.link(predecessor, successor);
            if predecessor == 0 {
                context.set_head(mv.route_x, successor);
            }
            Self::segment_insertion(
                solution, context,
                mv.left_x, mv.right_x,
                mv.left_y, mv.right_y,
                mv.route_y, mv.direction_x,
            );
        } else {
            // Full swap
            let predecessor_x = solution.predecessor(mv.left_x);
            let successor_x = solution.successor(mv.right_x);
            let predecessor_y = solution.predecessor(mv.left_y);
            let successor_y = solution.successor(mv.right_y);

            Self::segment_insertion(
                solution, context,
                mv.left_x, mv.right_x,
                predecessor_y, successor_y,
                mv.route_y, mv.direction_x,
            );
            Self::segment_insertion(
                solution, context,
                mv.left_y, mv.right_y,
                predecessor_x, successor_x,
                mv.route_x, mv.direction_y,
            );
        }
    }

    /// Evaluates a shift move (NUM_Y == 0).
    #[allow(clippy::too_many_arguments)]
    fn update_shift(
        instance: &Instance,
        solution: &AlkaidSolution,
        route_x: Node,
        route_y: Node,
        left: Node,
        right: Node,
        predecessor: Node,
        successor: Node,
        base_x: i32,
        cache: &mut BaseCache<SwapMove<NUM_X, NUM_Y>>,
        random: &mut Random,
    ) {
        let customer_left = solution.customer(left);
        let customer_predecessor = solution.customer(predecessor);
        let customer_right = solution.customer(right);
        let customer_successor = solution.customer(successor);

        let d1 = instance.distance(customer_left, customer_predecessor)
            + instance.distance(customer_right, customer_successor);
        let d2 = instance.distance(customer_left, customer_successor)
            + instance.distance(customer_right, customer_predecessor);

        let direction = if d1 >= d2 { 1 } else { 0 };
        let delta = base_x + (if direction != 0 { d2 } else { d1 })
            - instance.distance(customer_predecessor, customer_successor);

        if cache.delta.update(delta, random) {
            cache.mv = SwapMove {
                route_x,
                route_y,
                direction_x: direction,
                direction_y: -1,
                left_x: left,
                left_y: predecessor,
                right_x: right,
                right_y: successor,
            };
        }
    }

    /// Evaluates a full swap move.
    #[allow(clippy::too_many_arguments)]
    fn update_swap(
        instance: &Instance,
        solution: &AlkaidSolution,
        route_x: Node,
        route_y: Node,
        left_x: Node,
        right_x: Node,
        left_y: Node,
        right_y: Node,
        base_x: i32,
        cache: &mut BaseCache<SwapMove<NUM_X, NUM_Y>>,
        random: &mut Random,
    ) {
        let customer_left_x = solution.customer(left_x);
        let customer_right_x = solution.customer(right_x);
        let customer_left_y = solution.customer(left_y);
        let customer_right_y = solution.customer(right_y);
        let predecessor_x = solution.customer(solution.predecessor(left_x));
        let successor_x = solution.customer(solution.successor(right_x));
        let predecessor_y = solution.customer(solution.predecessor(left_y));
        let successor_y = solution.customer(solution.successor(right_y));

        let d1 = instance.distance(customer_left_x, predecessor_y)
            + instance.distance(customer_right_x, successor_y);
        let d2 = instance.distance(customer_left_x, successor_y)
            + instance.distance(customer_right_x, predecessor_y);
        let d3 = instance.distance(customer_left_y, predecessor_x)
            + instance.distance(customer_right_y, successor_x);
        let d4 = instance.distance(customer_left_y, successor_x)
            + instance.distance(customer_right_y, predecessor_x);

        let direction_x = if d1 >= d2 { 1 } else { 0 };
        let direction_y = if d3 >= d4 { 1 } else { 0 };

        let delta = base_x
            + (if direction_x != 0 { d2 } else { d1 })
            + (if direction_y != 0 { d4 } else { d3 })
            - instance.distance(customer_left_y, predecessor_y)
            - instance.distance(customer_right_y, successor_y);

        if cache.delta.update(delta, random) {
            cache.mv = SwapMove {
                route_x,
                route_y,
                direction_x,
                direction_y,
                left_x,
                left_y,
                right_x,
                right_y,
            };
        }
    }

    /// Inner evaluation loop for swap moves.
    fn swap_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route_x: Node,
        route_y: Node,
        cache: &mut BaseCache<SwapMove<NUM_X, NUM_Y>>,
        random: &mut Random,
    ) {
        let mut left_x = context.head(route_x);
        let mut load_x = solution.load(left_x);
        let mut right_x = left_x;

        // Build initial segment of NUM_X nodes
        for _ in 1..NUM_X {
            if right_x == 0 {
                return;
            }
            right_x = solution.successor(right_x);
            if right_x != 0 {
                load_x += solution.load(right_x);
            }
        }

        while right_x != 0 {
            let base_x = -instance.distance(
                solution.customer(left_x),
                solution.customer(solution.predecessor(left_x)),
            ) - instance.distance(
                solution.customer(right_x),
                solution.customer(solution.successor(right_x)),
            );

            let base_x = if NUM_Y == 0 {
                base_x + instance.distance(
                    solution.customer(solution.predecessor(left_x)),
                    solution.customer(solution.successor(right_x)),
                )
            } else {
                base_x
            };

            let load_y_lower = -instance.capacity + context.load(route_y) + load_x;

            if NUM_Y == 0 {
                if load_y_lower <= 0 {
                    let mut predecessor = 0;
                    let mut successor = context.head(route_y);
                    loop {
                        Self::update_shift(
                            instance, solution,
                            route_x, route_y,
                            left_x, right_x,
                            predecessor, successor,
                            base_x, cache, random,
                        );
                        if successor == 0 {
                            break;
                        }
                        predecessor = successor;
                        successor = solution.successor(successor);
                    }
                }
            } else {
                let load_y_upper = instance.capacity - context.load(route_x) + load_x;

                let mut left_y = context.head(route_y);
                let mut load_y = solution.load(left_y);
                let mut right_y = left_y;

                for _ in 1..NUM_Y {
                    if right_y == 0 {
                        break;
                    }
                    right_y = solution.successor(right_y);
                    if right_y != 0 {
                        load_y += solution.load(right_y);
                    }
                }

                while right_y != 0 {
                    if load_y >= load_y_lower && load_y <= load_y_upper {
                        Self::update_swap(
                            instance, solution,
                            route_x, route_y,
                            left_x, right_x,
                            left_y, right_y,
                            base_x, cache, random,
                        );
                    }
                    load_y -= solution.load(left_y);
                    left_y = solution.successor(left_y);
                    right_y = solution.successor(right_y);
                    if right_y != 0 {
                        load_y += solution.load(right_y);
                    }
                }
            }

            load_x -= solution.load(left_x);
            left_x = solution.successor(left_x);
            right_x = solution.successor(right_x);
            if right_x != 0 {
                load_x += solution.load(right_x);
            }
        }
    }
}

impl<const NUM_X: usize, const NUM_Y: usize> InterOperator for Swap<NUM_X, NUM_Y>
where
    SwapMove<NUM_X, NUM_Y>: Clone + Default,
{
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node> {
        let caches: &mut InterRouteCache<SwapMove<NUM_X, NUM_Y>> = cache_map.get(solution, context);

        let mut best_move = SwapMove::default();
        let mut best_delta = Delta::default();

        for route_x in 0..context.num_routes() {
            let start_y = if NUM_X != NUM_Y { 0 } else { route_x + 1 };
            for route_y in start_y..context.num_routes() {
                if NUM_X != NUM_Y && route_x == route_y {
                    continue;
                }

                let cache = caches.get(route_x, route_y);
                if !cache.try_reuse() {
                    Self::swap_inner(instance, solution, context, route_x, route_y, cache, random);
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
            Self::do_swap(&best_move, solution, context);
            vec![best_move.route_x, best_move.route_y]
        } else {
            vec![]
        }
    }
}
