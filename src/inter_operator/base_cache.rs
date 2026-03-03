//! Base cache for inter-route operators.
//!
//! Provides caching infrastructure to avoid redundant computations
//! when routes haven't changed.

use crate::cache::Cache;
use crate::delta::Delta;
use crate::instance::Node;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use std::any::Any;

/// Single cache entry storing a move and its delta.
///
/// # Type Parameters
///
/// * `T` - The move type
#[derive(Clone)]
pub struct BaseCache<T: Clone + Default> {
    /// Whether this cache entry is invalidated
    pub invalidated: bool,

    /// The cached delta value
    pub delta: Delta<i32>,

    /// The cached move
    pub mv: T,
}

impl<T: Clone + Default> Default for BaseCache<T> {
    fn default() -> Self {
        Self {
            invalidated: true,
            delta: Delta::default(),
            mv: T::default(),
        }
    }
}

impl<T: Clone + Default> BaseCache<T> {
    /// Attempts to reuse the cached value.
    ///
    /// # Returns
    ///
    /// `true` if the cache was valid and can be reused, `false` if it was invalidated
    pub fn try_reuse(&mut self) -> bool {
        if !self.invalidated {
            return true;
        }
        self.invalidated = false;
        self.delta = Delta::default();
        false
    }
}

/// Cache for inter-route operator results.
///
/// Stores a matrix of cached moves between route pairs.
///
/// # Type Parameters
///
/// * `T` - The move type
pub struct InterRouteCache<T: Clone + Default + 'static> {
    /// Cache matrix indexed by [route_a][route_b]
    matrix: Vec<Vec<BaseCache<T>>>,

    /// Mapping from route indices to matrix indices
    route_index_mappings: Vec<Node>,

    /// Pool of active route indices
    route_pool: Vec<Node>,

    /// Indices available for reuse
    unused_indices: Vec<Node>,

    /// Maximum index currently allocated
    max_index: Node,
}

impl<T: Clone + Default + 'static> Default for InterRouteCache<T> {
    fn default() -> Self {
        Self {
            matrix: Vec::new(),
            route_index_mappings: Vec::new(),
            route_pool: Vec::new(),
            unused_indices: Vec::new(),
            max_index: 0,
        }
    }
}

impl<T: Clone + Default + 'static> InterRouteCache<T> {
    /// Gets the cache entry for a pair of routes.
    #[inline]
    pub fn get(&mut self, route_a: Node, route_b: Node) -> &mut BaseCache<T> {
        let idx_a = self.route_index_mappings[route_a as usize] as usize;
        let idx_b = self.route_index_mappings[route_b as usize] as usize;
        &mut self.matrix[idx_a][idx_b]
    }
}

impl<T: Clone + Default + 'static> Cache for InterRouteCache<T> {
    fn reset(&mut self, _solution: &AlkaidSolution, context: &RouteContext) {
        self.max_index = context.num_routes();
        self.matrix.resize(self.max_index as usize, Vec::new());
        self.route_index_mappings.resize(self.max_index as usize, 0);
        self.route_pool.clear();
        self.unused_indices.clear();

        for i in 0..context.num_routes() {
            self.matrix[i as usize].resize(context.num_routes() as usize, BaseCache::default());
            self.route_index_mappings[i as usize] = i;
            self.route_pool.push(i);

            for j in 0..context.num_routes() {
                self.matrix[i as usize][j as usize].invalidated = true;
            }
        }
    }

    fn add_route(&mut self, route_index: Node) {
        let index = if let Some(idx) = self.unused_indices.pop() {
            idx
        } else {
            let idx = self.max_index;
            self.max_index += 1;
            self.route_index_mappings.resize(self.max_index as usize, 0);
            self.matrix.resize(self.max_index as usize, Vec::new());
            for i in 0..self.max_index as usize {
                self.matrix[i].resize(self.max_index as usize, BaseCache::default());
            }
            idx
        };

        self.route_index_mappings[route_index as usize] = index;
        self.route_pool.push(index);

        for &other in &self.route_pool {
            self.matrix[index as usize][other as usize].invalidated = true;
            self.matrix[other as usize][index as usize].invalidated = true;
        }
    }

    fn remove_route(&mut self, route_index: Node) {
        let index = self.route_index_mappings[route_index as usize];
        if let Some(pos) = self.route_pool.iter().position(|&x| x == index) {
            self.route_pool.swap_remove(pos);
        }
        self.unused_indices.push(index);
    }

    fn move_route(&mut self, dest_route_index: Node, src_route_index: Node) {
        self.route_index_mappings[dest_route_index as usize] =
            self.route_index_mappings[src_route_index as usize];
    }

    fn save(&mut self, _solution: &AlkaidSolution, _context: &RouteContext) {
        // No-op for this cache type
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
