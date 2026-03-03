//! Solution representation for the Split Delivery Vehicle Routing Problem.
//!
//! The solution uses a linked-list style representation where each node
//! stores predecessor/successor links, customer ID, and load. This enables
//! O(1) insertions and deletions within routes.

use crate::instance::{Instance, Node};
use std::fmt;

/// Internal data for a single node in the solution.
///
/// Each node represents a visit to a customer with a specific load.
/// Multiple nodes can serve the same customer (split delivery).
#[derive(Debug, Clone, Default)]
struct NodeData {
    /// Index of the successor node (0 = depot/end of route)
    successor: Node,

    /// Index of the predecessor node (0 = depot/start of route)
    predecessor: Node,

    /// Customer index being served by this node
    customer: Node,

    /// Load delivered to this customer at this node
    load: i32,

    /// Index of this node in the used_nodes vector (for O(1) removal)
    index_in_used_nodes: usize,
}

/// Solution representation for the SDVRP.
///
/// Uses a pool of nodes with linked-list style connections between them.
/// Routes are implicitly defined by chains of nodes starting from nodes
/// whose predecessor is 0 (depot).
///
/// # Design
///
/// - Node 0 is reserved for the depot
/// - Nodes are allocated from a pool and recycled when removed
/// - Each node stores links to its predecessor and successor
/// - Routes are chains: depot → node₁ → node₂ → ... → depot
///
/// # Example
///
/// ```
/// use alkaidsd::solution::AlkaidSolution;
///
/// let mut solution = AlkaidSolution::new();
///
/// // Insert a new node serving customer 1 with load 50
/// let node = solution.insert(1, 50, 0, 0);
///
/// // Check the structure
/// assert_eq!(solution.predecessor(node), 0);  // from depot
/// assert_eq!(solution.successor(node), 0);    // to depot
/// ```
#[derive(Debug, Clone)]
pub struct AlkaidSolution {
    /// Storage for all node data (node 0 is depot)
    node_data: Vec<NodeData>,

    /// Indices of nodes currently in use
    used_nodes: Vec<Node>,

    /// Indices of nodes available for reuse
    unused_nodes: Vec<Node>,
}

impl Default for AlkaidSolution {
    fn default() -> Self {
        Self::new()
    }
}

impl AlkaidSolution {
    /// Creates a new empty solution with only the depot node.
    #[inline]
    pub fn new() -> Self {
        let mut solution = Self {
            node_data: Vec::new(),
            used_nodes: Vec::new(),
            unused_nodes: Vec::new(),
        };
        // Add depot node
        solution.node_data.push(NodeData::default());
        solution
    }

    // ===== Getters =====

    /// Returns the predecessor of the given node.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The node to query
    ///
    /// # Returns
    ///
    /// The predecessor node index (0 = depot/start of route)
    #[inline]
    pub fn predecessor(&self, node_index: Node) -> Node {
        unsafe { self.node_data.get_unchecked(node_index as usize).predecessor }
    }

    /// Returns the successor of the given node.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The node to query
    ///
    /// # Returns
    ///
    /// The successor node index (0 = depot/end of route)
    #[inline]
    pub fn successor(&self, node_index: Node) -> Node {
        unsafe { self.node_data.get_unchecked(node_index as usize).successor }
    }

    /// Returns the customer served by the given node.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The node to query
    #[inline]
    pub fn customer(&self, node_index: Node) -> Node {
        unsafe { self.node_data.get_unchecked(node_index as usize).customer }
    }

    /// Returns the load delivered at the given node.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The node to query
    #[inline]
    pub fn load(&self, node_index: Node) -> i32 {
        unsafe { self.node_data.get_unchecked(node_index as usize).load }
    }

    // ===== Setters =====

    /// Sets the predecessor of a node.
    #[inline]
    pub fn set_predecessor(&mut self, node_index: Node, predecessor: Node) {
        self.node_data[node_index as usize].predecessor = predecessor;
    }

    /// Sets the successor of a node.
    #[inline]
    pub fn set_successor(&mut self, node_index: Node, successor: Node) {
        self.node_data[node_index as usize].successor = successor;
    }

    /// Sets the customer of a node.
    #[inline]
    pub fn set_customer(&mut self, node_index: Node, customer: Node) {
        self.node_data[node_index as usize].customer = customer;
    }

    /// Sets the load of a node.
    #[inline]
    pub fn set_load(&mut self, node_index: Node, load: i32) {
        self.node_data[node_index as usize].load = load;
    }

    // ===== Structure Operations =====

    /// Links two nodes together (predecessor → successor).
    ///
    /// # Arguments
    ///
    /// * `predecessor` - The predecessor node
    /// * `successor` - The successor node
    #[inline]
    pub fn link(&mut self, predecessor: Node, successor: Node) {
        self.set_predecessor(successor, predecessor);
        self.set_successor(predecessor, successor);
    }

    /// Removes a node from its current position.
    ///
    /// The node is moved to the unused pool and its predecessor/successor
    /// are linked together.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The node to remove
    pub fn remove(&mut self, node_index: Node) {
        let predecessor = self.predecessor(node_index);
        let successor = self.successor(node_index);
        self.link(predecessor, successor);

        // Move to unused pool using swap-remove for O(1)
        let index_in_used = self.node_data[node_index as usize].index_in_used_nodes;
        let last_node = *self.used_nodes.last().unwrap();
        self.node_data[last_node as usize].index_in_used_nodes = index_in_used;
        self.used_nodes[index_in_used] = last_node;
        self.used_nodes.pop();
        self.unused_nodes.push(node_index);
    }

    /// Inserts a new node between predecessor and successor.
    ///
    /// # Arguments
    ///
    /// * `customer` - The customer to serve
    /// * `load` - The load to deliver
    /// * `predecessor` - The predecessor node
    /// * `successor` - The successor node
    ///
    /// # Returns
    ///
    /// The index of the newly created node
    #[inline]
    pub fn insert(&mut self, customer: Node, load: i32, predecessor: Node, successor: Node) -> Node {
        let node_index = self.new_node(customer, load);
        self.link(predecessor, node_index);
        self.link(node_index, successor);
        node_index
    }

    /// Creates a new node with the given customer and load.
    ///
    /// Reuses nodes from the unused pool if available.
    ///
    /// # Arguments
    ///
    /// * `customer` - The customer to serve
    /// * `load` - The load to deliver
    ///
    /// # Returns
    ///
    /// The index of the new node
    pub fn new_node(&mut self, customer: Node, load: i32) -> Node {
        let node_index = if let Some(idx) = self.unused_nodes.pop() {
            idx
        } else {
            let idx = self.node_data.len() as Node;
            self.node_data.push(NodeData::default());
            idx
        };

        self.node_data[node_index as usize].index_in_used_nodes = self.used_nodes.len();
        self.used_nodes.push(node_index);
        self.set_customer(node_index, customer);
        self.set_load(node_index, load);
        node_index
    }

    /// Returns all used node indices.
    #[inline]
    pub fn node_indices(&self) -> &[Node] {
        &self.used_nodes
    }

    /// Returns the maximum node index in use.
    #[inline]
    pub fn max_node_index(&self) -> Node {
        (self.node_data.len() - 1) as Node
    }

    /// Reverses the links between two nodes and connects to new neighbors.
    ///
    /// Used for route segment reversal operations (e.g., 2-opt style moves).
    ///
    /// # Arguments
    ///
    /// * `left` - First node of the segment
    /// * `right` - Last node of the segment
    /// * `predecessor` - New predecessor for the segment
    /// * `successor` - New successor for the segment
    pub fn reversed_link(&mut self, left: Node, right: Node, predecessor: Node, successor: Node) {
        let mut current = right;
        let mut new_pred = predecessor;

        loop {
            let original_predecessor = self.predecessor(current);
            self.link(new_pred, current);
            if current == left {
                break;
            }
            new_pred = current;
            current = original_predecessor;
        }
        self.link(left, successor);
    }

    /// Calculates the objective value (total distance) of the solution.
    ///
    /// # Arguments
    ///
    /// * `instance` - The problem instance
    ///
    /// # Returns
    ///
    /// The total distance traveled by all routes
    pub fn calc_objective(&self, instance: &Instance) -> i32 {
        let mut objective = 0;

        for &node_index in self.node_indices() {
            let predecessor = self.predecessor(node_index);
            let successor = self.successor(node_index);

            // Distance from predecessor to this node
            objective += instance.distance(self.customer(predecessor), self.customer(node_index));

            // If this is the last node in route, add distance back to depot
            if successor == 0 {
                objective += instance.distance(self.customer(node_index), 0);
            }
        }

        objective
    }

    /// Outputs the solution in JSON format.
    ///
    /// # Returns
    ///
    /// A JSON string representation of all routes
    pub fn to_json(&self) -> String {
        let mut routes = Vec::new();

        for &node_index in self.node_indices() {
            if self.predecessor(node_index) == 0 {
                // Start of a new route
                let mut route = Vec::new();
                route.push(r#"{ "customer": 0, "quantity": 0 }"#.to_string());

                let mut current = node_index;
                while current != 0 {
                    route.push(format!(
                        r#"{{ "customer": {}, "quantity": {} }}"#,
                        self.customer(current),
                        self.load(current)
                    ));
                    current = self.successor(current);
                }

                route.push(r#"{ "customer": 0, "quantity": 0 }"#.to_string());
                routes.push(format!("[{}]", route.join(", ")));
            }
        }

        format!("[{}]", routes.join(",\n"))
    }
}

impl fmt::Display for AlkaidSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut num_routes = 0;

        for &node_index in self.node_indices() {
            if self.predecessor(node_index) == 0 {
                num_routes += 1;
                write!(f, "Route {}: 0", num_routes)?;

                let mut current = node_index;
                while current != 0 {
                    write!(
                        f,
                        " - {} ( {} )",
                        self.customer(current),
                        self.load(current)
                    )?;
                    current = self.successor(current);
                }
                writeln!(f, " - 0")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solution_insert_remove() {
        let mut solution = AlkaidSolution::new();

        // Insert a node
        let node1 = solution.insert(1, 50, 0, 0);
        assert_eq!(solution.predecessor(node1), 0);
        assert_eq!(solution.successor(node1), 0);
        assert_eq!(solution.customer(node1), 1);
        assert_eq!(solution.load(node1), 50);

        // Insert another node after node1
        let node2 = solution.insert(2, 30, node1, 0);
        assert_eq!(solution.successor(node1), node2);
        assert_eq!(solution.predecessor(node2), node1);

        // Remove node1
        solution.remove(node1);
        assert_eq!(solution.predecessor(node2), 0);
    }

    #[test]
    fn test_solution_calc_objective() {
        let instance = Instance {
            num_customers: 3,
            capacity: 100,
            demands: vec![0, 50, 30],
            distance_matrix: vec![
                vec![0, 10, 20],
                vec![10, 0, 15],
                vec![20, 15, 0],
            ],
        };

        let mut solution = AlkaidSolution::new();
        let node1 = solution.insert(1, 50, 0, 0);
        let _node2 = solution.insert(2, 30, node1, 0);

        // Route: 0 -> 1 -> 2 -> 0
        // Distance: 10 + 15 + 20 = 45
        assert_eq!(solution.calc_objective(&instance), 45);
    }
}
