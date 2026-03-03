//! Problem instance definition for the Split Delivery Vehicle Routing Problem.
//!
//! The SDVRP instance consists of:
//! - A set of customers (nodes), each with a demand
//! - A depot (node 0) with zero demand
//! - Vehicle capacity constraint
//! - Distance matrix between all node pairs

/// Node index type. Using i16 for memory efficiency while supporting
/// up to 32,767 customers, which is sufficient for practical instances.
/// This mirrors the C++ `using Node = short` declaration.
pub type Node = i16;

/// Represents a problem instance for the Split Delivery Vehicle Routing Problem.
///
/// # Fields
///
/// * `num_customers` - Total number of nodes including the depot (node 0)
/// * `capacity` - Maximum load capacity of each vehicle
/// * `demands` - Demand at each customer location (depot has zero demand)
/// * `distance_matrix` - Symmetric distance/cost matrix between all nodes
///
/// # Example
///
/// ```
/// use alkaidsd::Instance;
///
/// let instance = Instance {
///     num_customers: 3,  // depot + 2 customers
///     capacity: 100,
///     demands: vec![0, 50, 75],  // depot=0, customer1=50, customer2=75
///     distance_matrix: vec![
///         vec![0, 10, 20],
///         vec![10, 0, 15],
///         vec![20, 15, 0],
///     ],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Instance {
    /// The number of customers, including the depot (node 0).
    pub num_customers: Node,

    /// The capacity of each vehicle.
    pub capacity: i32,

    /// The demands of each customer, including the depot (which has zero demand).
    /// Index 0 represents the depot, indices 1..num_customers represent customers.
    pub demands: Vec<i32>,

    /// The distance/cost matrix between customers, including the depot.
    /// `distance_matrix[i][j]` gives the cost of traveling from node i to node j.
    pub distance_matrix: Vec<Vec<i32>>,
}

impl Instance {
    /// Creates a new instance with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `num_customers` - Number of nodes including depot
    /// * `capacity` - Vehicle capacity
    /// * `demands` - Demand vector
    /// * `distance_matrix` - Distance matrix
    #[inline]
    pub fn new(
        num_customers: Node,
        capacity: i32,
        demands: Vec<i32>,
        distance_matrix: Vec<Vec<i32>>,
    ) -> Self {
        Self {
            num_customers,
            capacity,
            demands,
            distance_matrix,
        }
    }

    /// Returns the distance between two nodes.
    ///
    /// # Arguments
    ///
    /// * `from` - Source node index
    /// * `to` - Destination node index
    ///
    /// # Safety
    ///
    /// Uses unchecked indexing for performance since this is the hottest
    /// function in the solver (called millions of times per second).
    /// Indices are always valid customer IDs within 0..num_customers.
    #[inline]
    pub fn distance(&self, from: Node, to: Node) -> i32 {
        unsafe {
            *self.distance_matrix
                .get_unchecked(from as usize)
                .get_unchecked(to as usize)
        }
    }

    /// Returns the demand at a given node.
    ///
    /// # Arguments
    ///
    /// * `node` - Node index
    #[inline]
    pub fn demand(&self, node: Node) -> i32 {
        self.demands[node as usize]
    }
}
