//! Relocate operator implementation.
//!
//! Moves a single node from one route to another using star cache optimization.

use super::base_cache::{BaseCache, InterRouteCache};
use super::base_star::StarCaches;
use super::{calc_delta, InterOperator};
use crate::cache::CacheMap;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Move data for the Relocate operator.
#[derive(Clone, Default)]
struct RelocateMove {
    route_x: Node,
    route_y: Node,
    node_x: Node,
    predecessor_x: Node,
    successor_x: Node,
}

/// Relocate operator.
///
/// Moves a single node from route X to route Y, finding the best
/// insertion position using star cache optimization.
#[derive(Debug, Clone, Default)]
pub struct Relocate;

impl Relocate {
    /// Applies a relocate move.
    fn do_relocate(mv: &RelocateMove, solution: &mut AlkaidSolution, context: &mut RouteContext) {
        let predecessor_x = solution.predecessor(mv.node_x);
        let successor_x = solution.successor(mv.node_x);

        // Remove from route X
        solution.set_successor(0, context.head(mv.route_x));
        solution.link(predecessor_x, successor_x);
        context.set_head(mv.route_x, solution.successor(0));

        // Insert into route Y
        solution.set_successor(0, context.head(mv.route_y));
        solution.link(mv.predecessor_x, mv.node_x);
        solution.link(mv.node_x, mv.successor_x);
        context.set_head(mv.route_y, solution.successor(0));
    }

    /// Inner function to evaluate all possible relocates from route_x to route_y.
    #[allow(clippy::too_many_arguments)]
    fn relocate_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route_x: Node,
        route_y: Node,
        cache: &mut BaseCache<RelocateMove>,
        star_caches: &StarCaches,
        random: &mut Random,
    ) {
        let mut node_x = context.head(route_x);
        while node_x != 0 {
            // Check capacity constraint
            if context.load(route_y) + solution.load(node_x) <= instance.capacity {
                // Use star cache to find best insertion position
                let insertion = star_caches.get(route_y, solution.customer(node_x)).find_best();
                let predecessor_x = solution.predecessor(node_x);
                let successor_x = solution.successor(node_x);
                    
                let delta = insertion.delta.value
                    - calc_delta(instance, solution, node_x, predecessor_x, successor_x);
                    
                if cache.delta.update(delta, random) {
                    cache.mv = RelocateMove {
                        route_x,
                        route_y,
                        node_x,
                        predecessor_x: insertion.predecessor,
                        successor_x: insertion.successor,
                    };
                }
            }
            node_x = solution.successor(node_x);
        }
    }
}

impl InterOperator for Relocate {
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node> {
        let (caches, star_caches) = cache_map
            .get2_mut::<InterRouteCache<RelocateMove>, StarCaches>(solution, context);
        let mut best_move = RelocateMove::default();
        let mut best_delta = Delta::default();

        for route_x in 0..context.num_routes() {
            for route_y in 0..context.num_routes() {
                if route_x == route_y {
                    continue;
                }
                let cache = caches.get(route_x, route_y);
                if !cache.try_reuse() {
                    star_caches.preprocess(instance, solution, context, route_y, random);
                    Self::relocate_inner(
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
            Self::do_relocate(&best_move, solution, context);
            vec![best_move.route_x, best_move.route_y]
        } else {
            vec![]
        }
    }
}
