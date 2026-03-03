//! Route context for efficient route queries.
//!
//! The RouteContext maintains metadata about routes including head/tail nodes,
//! total load, and cumulative load at each position. This enables O(1) queries
//! for route properties during optimization.

use crate::instance::Node;
use crate::solution::AlkaidSolution;

/// Metadata for a single route.
#[derive(Debug, Clone, Default)]
struct RouteData {
    /// First node in the route (after depot)
    head: Node,

    /// Last node in the route (before depot)
    tail: Node,

    /// Total load in the route
    load: i32,
}

/// Context structure for efficient route operations.
///
/// Maintains route metadata to avoid recomputation during optimization.
/// Must be kept synchronized with the solution structure.
///
/// # Example
///
/// ```
/// use alkaidsd::route_context::RouteContext;
/// use alkaidsd::solution::AlkaidSolution;
///
/// let solution = AlkaidSolution::new();
/// let mut context = RouteContext::new();
/// context.calc_route_context(&solution);
/// ```
#[derive(Debug, Clone, Default)]
pub struct RouteContext {
    /// Route metadata for each route
    routes: Vec<RouteData>,

    /// Cumulative load before each node
    pre_loads: Vec<i32>,
}

impl RouteContext {
    /// Creates a new empty route context.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the head (first) node of the given route.
    ///
    /// # Arguments
    ///
    /// * `route_index` - Index of the route
    #[inline]
    pub fn head(&self, route_index: Node) -> Node {
        self.routes[route_index as usize].head
    }

    /// Returns the tail (last) node of the given route.
    ///
    /// # Arguments
    ///
    /// * `route_index` - Index of the route
    #[inline]
    pub fn tail(&self, route_index: Node) -> Node {
        self.routes[route_index as usize].tail
    }

    /// Returns the total load of the given route.
    ///
    /// # Arguments
    ///
    /// * `route_index` - Index of the route
    #[inline]
    pub fn load(&self, route_index: Node) -> i32 {
        self.routes[route_index as usize].load
    }

    /// Returns the cumulative load before the given node.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The node to query
    #[inline]
    pub fn pre_load(&self, node_index: Node) -> i32 {
        unsafe { *self.pre_loads.get_unchecked(node_index as usize) }
    }

    /// Sets the head node of a route.
    ///
    /// # Arguments
    ///
    /// * `route_index` - Index of the route
    /// * `head` - New head node
    #[inline]
    pub fn set_head(&mut self, route_index: Node, head: Node) {
        self.routes[route_index as usize].head = head;
    }

    /// Adds load to a route's total.
    ///
    /// # Arguments
    ///
    /// * `route_index` - Index of the route
    /// * `load` - Load to add
    #[inline]
    pub fn add_load(&mut self, route_index: Node, load: i32) {
        self.routes[route_index as usize].load += load;
    }

    /// Returns the number of routes.
    #[inline]
    pub fn num_routes(&self) -> Node {
        self.routes.len() as Node
    }

    /// Sets the number of routes.
    #[inline]
    pub fn set_num_routes(&mut self, num_routes: Node) {
        self.routes.resize(num_routes as usize, RouteData::default());
    }

    /// Adds a new route with the given head, tail, and load.
    ///
    /// # Arguments
    ///
    /// * `head` - First node of the route
    /// * `tail` - Last node of the route
    /// * `load` - Total load of the route
    #[inline]
    pub fn add_route(&mut self, head: Node, tail: Node, load: i32) {
        self.routes.push(RouteData { head, tail, load });
    }

    /// Recalculates all route context from the solution.
    ///
    /// Should be called when the solution structure changes significantly.
    ///
    /// # Arguments
    ///
    /// * `solution` - The solution to analyze
    pub fn calc_route_context(&mut self, solution: &AlkaidSolution) {
        self.routes.clear();

        // Find all route heads (nodes with predecessor = depot)
        for &node_index in solution.node_indices() {
            if solution.predecessor(node_index) == 0 {
                self.routes.push(RouteData {
                    head: node_index,
                    tail: node_index, // Will be updated
                    load: 0,          // Will be updated
                });
            }
        }

        // Resize pre_loads to accommodate all nodes
        self.pre_loads
            .resize((solution.max_node_index() + 1) as usize, 0);

        // Update context for each route
        let num_routes = self.num_routes();
        for route_index in 0..num_routes {
            self.update_route_context(solution, route_index, 0);
        }
    }

    /// Updates route context starting from a given predecessor.
    ///
    /// Call this after modifying a route to update cumulative loads and tail.
    ///
    /// # Arguments
    ///
    /// * `solution` - The solution
    /// * `route_index` - Index of the route to update
    /// * `predecessor` - Node to start updating from (0 for route start)
    pub fn update_route_context(&mut self, solution: &AlkaidSolution, route_index: Node, predecessor: Node) {
        // Ensure pre_loads is large enough
        let required_size = (solution.max_node_index() + 1) as usize;
        if self.pre_loads.len() < required_size {
            self.pre_loads.resize(required_size, 0);
        }

        // When starting from depot (predecessor == 0), the cumulative load is 0.
        // When starting from a node, we use its stored cumulative load.
        let mut load: i32 = self.pre_loads.get(predecessor as usize).copied().unwrap_or(0);
        
        let mut node_index = if predecessor != 0 {
            solution.successor(predecessor)
        } else {
            self.head(route_index)
        };

        let mut last_node = predecessor;
        while node_index != 0 {
            let node_load = solution.load(node_index);
            // In a valid solution, total load per route should never exceed capacity.
            // A debug assertion catches overflow during development.
            load += node_load;
            debug_assert!(load >= 0, "route load underflow detected");
            self.pre_loads[node_index as usize] = load;
            last_node = node_index;
            node_index = solution.successor(node_index);
        }

        self.routes[route_index as usize].tail = last_node;
        self.routes[route_index as usize].load = load;
    }

    /// Moves route context from source to destination route index.
    ///
    /// Used when compacting routes after removal.
    ///
    /// # Arguments
    ///
    /// * `dest_route_index` - Destination route index
    /// * `src_route_index` - Source route index
    #[inline]
    pub fn move_route_context(&mut self, dest_route_index: Node, src_route_index: Node) {
        self.routes[dest_route_index as usize] = self.routes[src_route_index as usize].clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_context_basic() {
        let mut solution = AlkaidSolution::new();
        let node1 = solution.insert(1, 50, 0, 0);
        let _node2 = solution.insert(2, 30, node1, 0);

        let mut context = RouteContext::new();
        context.calc_route_context(&solution);

        assert_eq!(context.num_routes(), 1);
        assert_eq!(context.load(0), 80);
    }

    #[test]
    fn test_route_context_multiple_routes() {
        let mut solution = AlkaidSolution::new();

        // First route
        let node1 = solution.insert(1, 50, 0, 0);
        solution.insert(2, 30, node1, 0);

        // Second route (new route head)
        let node3 = solution.insert(3, 40, 0, 0);
        solution.insert(4, 20, node3, 0);

        let mut context = RouteContext::new();
        context.calc_route_context(&solution);

        assert_eq!(context.num_routes(), 2);
    }
}
