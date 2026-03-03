//! Inter-route operators for improving solutions across routes.
//!
//! These operators exchange or relocate nodes between different routes
//! to improve the overall solution quality.

pub mod base_cache;
pub mod base_star;
pub mod cross;
pub mod relocate;
pub mod sd_swap_one_one;
pub mod sd_swap_star;
pub mod sd_swap_two_one;
pub mod swap;
pub mod swap_star;

use crate::cache::CacheMap;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

// Re-exports
pub use cross::Cross;
pub use relocate::Relocate;
pub use sd_swap_one_one::SdSwapOneOne;
pub use sd_swap_star::SdSwapStar;
pub use sd_swap_two_one::SdSwapTwoOne;
pub use swap::Swap;
pub use swap_star::SwapStar;

/// Trait for inter-route operators.
///
/// Inter-route operators improve solutions by exchanging or relocating
/// nodes between different routes.
pub trait InterOperator {
    /// Attempts to improve the solution across routes.
    ///
    /// # Arguments
    ///
    /// * `instance` - The problem instance
    /// * `solution` - The solution to modify
    /// * `context` - Route context
    /// * `random` - Random number generator
    /// * `cache_map` - Cache for storing computed values
    ///
    /// # Returns
    ///
    /// A vector of modified route indices (empty if no improvement found)
    fn apply(
        &self,
        instance: &Instance,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) -> Vec<Node>;
}

/// Helper struct for managing route head during modifications.
///
/// Uses RAII to ensure route head is properly updated after modifications.
pub struct RouteHeadGuard<'a> {
    solution: &'a mut AlkaidSolution,
    context: &'a mut RouteContext,
    route_index: Node,
}

impl<'a> RouteHeadGuard<'a> {
    /// Creates a new route head guard.
    ///
    /// Sets up the depot's successor to point to the route head,
    /// allowing modifications that might change the head.
    pub fn new(
        solution: &'a mut AlkaidSolution,
        context: &'a mut RouteContext,
        route_index: Node,
    ) -> Self {
        solution.set_successor(0, context.head(route_index));
        Self {
            solution,
            context,
            route_index,
        }
    }

    /// Returns a mutable reference to the solution.
    #[inline]
    pub fn solution(&mut self) -> &mut AlkaidSolution {
        self.solution
    }
}

impl<'a> Drop for RouteHeadGuard<'a> {
    fn drop(&mut self) {
        // Restore route head from depot's successor
        self.context
            .set_head(self.route_index, self.solution.successor(0));
    }
}

/// Calculates the delta (change in cost) for inserting a node between two positions.
///
/// # Arguments
///
/// * `instance` - The problem instance
/// * `solution` - The solution
/// * `node_index` - The node to insert
/// * `predecessor` - The predecessor position
/// * `successor` - The successor position
#[inline]
pub fn calc_delta(
    instance: &Instance,
    solution: &AlkaidSolution,
    node_index: Node,
    predecessor: Node,
    successor: Node,
) -> i32 {
    instance.distance(solution.customer(node_index), solution.customer(predecessor))
        + instance.distance(solution.customer(node_index), solution.customer(successor))
        - instance.distance(solution.customer(predecessor), solution.customer(successor))
}
