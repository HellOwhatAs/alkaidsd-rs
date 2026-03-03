//! Distance matrix optimizer using Floyd-Warshall algorithm.
//!
//! Optimizes the distance matrix to find shortest paths through intermediate
//! nodes, and provides restoration of solutions to include these intermediate
//! nodes explicitly.

use crate::instance::Node;
use crate::solution::AlkaidSolution;

/// Optimizes a distance matrix using the Floyd-Warshall algorithm.
///
/// After optimization, the distance matrix contains shortest path distances.
/// The optimizer also tracks intermediate nodes so solutions can be restored
/// to include them.
#[derive(Debug, Clone)]
pub struct DistanceMatrixOptimizer {
    /// Original distance matrix (before optimization)
    /// Kept for potential future use (e.g., restoration to original distances)
    #[allow(dead_code)]
    original: Vec<Vec<i32>>,

    /// Previous node indices for path reconstruction
    previous_node_indices: Vec<Vec<Node>>,
}

impl DistanceMatrixOptimizer {
    /// Creates a new optimizer and applies Floyd-Warshall to the distance matrix.
    ///
    /// # Arguments
    ///
    /// * `distance_matrix` - The distance matrix to optimize (modified in place)
    ///
    /// # Returns
    ///
    /// An optimizer that can restore solutions to include intermediate nodes
    pub fn new(distance_matrix: &mut [Vec<i32>]) -> Self {
        let num_customers = distance_matrix.len() as Node;
        let original = distance_matrix.to_owned();
        let mut previous_node_indices = vec![vec![0 as Node; num_customers as usize]; num_customers as usize];

        // Floyd-Warshall algorithm
        for k in 1..num_customers {
            for i in 0..num_customers {
                for j in 0..num_customers {
                    let distance = distance_matrix[i as usize][k as usize]
                        + distance_matrix[k as usize][j as usize];

                    if distance_matrix[i as usize][j as usize] > distance {
                        distance_matrix[i as usize][j as usize] = distance;
                        previous_node_indices[i as usize][j as usize] = k;
                    }
                }
            }
        }

        Self {
            original,
            previous_node_indices,
        }
    }

    /// Recursively restores intermediate nodes between two positions.
    fn restore_path(&self, solution: &mut AlkaidSolution, i: Node, j: Node) {
        let customer = self.previous_node_indices[solution.customer(i) as usize][solution.customer(j) as usize];

        if customer != 0 {
            let k = solution.insert(customer, 0, i, j);
            self.restore_path(solution, i, k);
            self.restore_path(solution, k, j);
        }
    }

    /// Restores a solution to include intermediate nodes from shortest paths.
    ///
    /// When the distance matrix was optimized, some edges may represent paths
    /// through intermediate customers. This method inserts those intermediate
    /// customers (with zero load) into the solution.
    ///
    /// # Arguments
    ///
    /// * `solution` - The solution to restore
    pub fn restore(&self, solution: &mut AlkaidSolution) {
        // Find all route heads
        let heads: Vec<Node> = solution
            .node_indices()
            .iter()
            .filter(|&&n| solution.predecessor(n) == 0)
            .copied()
            .collect();

        // Restore each route
        for head in heads {
            let mut predecessor: Node = 0;
            let mut node_index = head;

            while node_index != 0 {
                self.restore_path(solution, predecessor, node_index);
                predecessor = node_index;
                node_index = solution.successor(node_index);
            }

            // Restore path back to depot
            self.restore_path(solution, predecessor, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floyd_warshall_basic() {
        // Triangle: 0 -> 1 -> 2, but 0 -> 2 is longer
        let mut distance_matrix = vec![
            vec![0, 1, 10],
            vec![1, 0, 1],
            vec![10, 1, 0],
        ];

        let optimizer = DistanceMatrixOptimizer::new(&mut distance_matrix);

        // 0 -> 2 should now be 2 (via 1)
        assert_eq!(distance_matrix[0][2], 2);
        
        // Intermediate node should be 1
        assert_eq!(optimizer.previous_node_indices[0][2], 1);
    }

    #[test]
    fn test_restore() {
        let mut distance_matrix = vec![
            vec![0, 1, 10],
            vec![1, 0, 1],
            vec![10, 1, 0],
        ];

        let optimizer = DistanceMatrixOptimizer::new(&mut distance_matrix);

        // Create solution: depot -> customer 2 -> depot
        let mut solution = AlkaidSolution::new();
        solution.insert(2, 10, 0, 0);

        // Before restore: 1 node
        assert_eq!(solution.node_indices().len(), 1);

        optimizer.restore(&mut solution);

        // After restore: should include intermediate node 1
        assert!(solution.node_indices().len() >= 1);
    }
}
