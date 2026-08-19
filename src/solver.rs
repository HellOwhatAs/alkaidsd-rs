//! Main solver implementation.
//!
//! Implements the Alkaid algorithm for solving Vehicle Routing Problems
//! using adaptive large neighborhood search. The solver is generic over
//! [`VrpVariant`], supporting multiple VRP variants (SDVRP, VRPPD, EVRP, etc.)
//! with zero-cost abstraction.
//!
//! # Usage
//!
//! For SDVRP (backward compatible):
//! ```rust,ignore
//! let solver = AlkaidSolver::default();
//! let solution = solver.solve(&mut config, &instance);
//! ```
//!
//! For other variants:
//! ```rust,ignore
//! let solver = AlkaidSolver::default();
//! let solution = solver.solve_variant::<MyVariant>(&mut config, &instance);
//! ```

use crate::acceptance_rule::AcceptanceRule;
use crate::cache::CacheMap;
use crate::instance::{Instance, Node};
use crate::inter_operator::InterOperator;
use crate::intra_operator::IntraOperator;
use crate::problem::VrpVariant;
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::ruin_method::RuinMethod;
use crate::sdvrp::Sdvrp;
use crate::solution::AlkaidSolution;
use crate::sorter::Sorter;
use std::time::Instant;

/// Listener trait for optimization events.
///
/// Implement this trait to receive callbacks during the optimization process.
pub trait Listener {
    /// Called when optimization starts.
    fn on_start(&mut self);

    /// Called when a new best solution is found.
    fn on_updated(&mut self, solution: &AlkaidSolution, objective: i32);

    /// Called when optimization ends.
    fn on_end(&mut self, solution: &AlkaidSolution, objective: i32);
}

/// Base configuration for solvers.
pub trait Config {
    /// Random seed for reproducibility.
    fn random_seed(&self) -> u32;

    /// Time limit in seconds.
    fn time_limit(&self) -> f64;
}

/// Trait for SDVRP solvers.
pub trait Solver {
    /// Solves the problem instance.
    fn solve(&self, config: &AlkaidConfig, instance: &Instance) -> AlkaidSolution;
}

/// Configuration for the Alkaid solver.
pub struct AlkaidConfig {
    /// Random seed for reproducibility
    pub random_seed: u32,

    /// Maximum number of iterations without improvement before termination
    pub max_stagnation: i32,

    /// Time limit in seconds
    pub time_limit: f64,

    /// Blink rate for split reinsertion (diversification)
    pub blink_rate: f64,

    /// Inter-route operators
    pub inter_operators: Vec<Box<dyn InterOperator>>,

    /// Intra-route operators
    pub intra_operators: Vec<Box<dyn IntraOperator>>,

    /// Factory function for acceptance rules
    pub acceptance_rule: Box<dyn AcceptanceRule>,

    /// Ruin method for perturbation
    pub ruin_method: Box<dyn RuinMethod>,

    /// Sorter for customer ordering
    pub sorter: Sorter,

    /// Optional listener for optimization events
    pub listener: Option<Box<dyn Listener>>,
}

/// The main Alkaid solver.
#[derive(Default)]
pub struct AlkaidSolver;

impl AlkaidSolver {
    /// Performs intra-route search on a single route.
    fn intra_route_search<V: VrpVariant>(
        instance: &V::Instance,
        config: &AlkaidConfig,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
    ) {
        V::repair(instance, route_index, solution, context);

        let base = V::base_instance(instance);
        let mut neighborhoods: Vec<usize> = (0..config.intra_operators.len()).collect();

        loop {
            random.shuffle(&mut neighborhoods);

            let mut improved = false;
            for &neighborhood in &neighborhoods {
                improved = config.intra_operators[neighborhood].apply(
                    base,
                    route_index,
                    solution,
                    context,
                    random,
                );
                if improved {
                    break;
                }
            }

            if !improved {
                break;
            }
        }
    }

    /// Performs randomized variable neighborhood descent across routes.
    fn randomized_variable_neighborhood_descent<V: VrpVariant>(
        instance: &V::Instance,
        config: &AlkaidConfig,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        cache_map: &mut CacheMap,
    ) {
        let base = V::base_instance(instance);
        cache_map.reset(solution, context);

        loop {
            let mut neighborhoods: Vec<usize> = (0..config.inter_operators.len()).collect();
            random.shuffle(&mut neighborhoods);

            let mut improved = false;

            for &neighborhood in &neighborhoods {
                let original_num_routes = context.num_routes();

                let routes = config.inter_operators[neighborhood]
                    .apply(base, solution, context, random, cache_map);

                if !routes.is_empty() {
                    let mut routes = routes;
                    routes.sort();
                    improved = true;

                    // Collect heads of modified routes
                    let mut heads = Vec::new();
                    for &route_index in &routes {
                        let head = context.head(route_index);
                        if head != 0 {
                            heads.push(head);
                        }
                        if route_index < original_num_routes {
                            cache_map.remove_route(route_index);
                        }
                    }

                    // Compact routes
                    let mut num_routes = 0;
                    for route_index in 0..context.num_routes() {
                        if !routes.contains(&route_index) {
                            context.move_route_context(num_routes, route_index);
                            cache_map.move_route(num_routes, route_index);
                            num_routes += 1;
                        }
                    }

                    // Re-add modified routes
                    for head in heads {
                        let required_capacity = (num_routes + 1) as usize;
                        if required_capacity > context.num_routes() as usize {
                            context.set_num_routes(num_routes + 1);
                        }
                        context.set_head(num_routes, head);
                        context.update_route_context(solution, num_routes, 0);
                        cache_map.add_route(num_routes);
                        Self::intra_route_search::<V>(
                            instance, config, num_routes, solution, context, random,
                        );
                        num_routes += 1;
                    }

                    context.set_num_routes(num_routes);
                    break;
                }
            }

            if !improved {
                break;
            }
        }

        cache_map.save(solution, context);
    }

    /// Performs perturbation by ruining and repairing.
    fn perturb<V: VrpVariant>(
        instance: &V::Instance,
        sorter: &Sorter,
        blink_rate: f64,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
        ruin_method: &mut dyn RuinMethod,
    ) {
        let base = V::base_instance(instance);
        context.calc_route_context(solution);

        // Ruin: get customers to remove
        let customers = ruin_method.ruin(base, solution, context, random);

        // Sort customers for reinsertion
        let mut customers = customers;
        sorter.sort(base, &mut customers, random);

        // Remove all nodes serving the selected customers (variant-specific)
        V::remove_customers(instance, &customers, solution, context);

        // Reinsert customers (variant-specific)
        V::reinsert_customers(instance, &customers, blink_rate, solution, context, random);
    }

    /// Solves a VRP instance using the specified variant.
    ///
    /// This is the generic solving method that works with any [`VrpVariant`].
    /// The variant determines how construction, repair, and reinsertion are
    /// performed, while operators work with the base [`Instance`] data.
    ///
    /// # Type Parameters
    ///
    /// * `V` - The VRP variant to solve (e.g., [`Sdvrp`])
    ///
    /// # Arguments
    ///
    /// * `config` - Solver configuration (operators, acceptance rule, etc.)
    /// * `instance` - The problem instance for the variant
    ///
    /// # Returns
    ///
    /// The best solution found within the time limit
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use alkaidsd::sdvrp::Sdvrp;
    ///
    /// let solver = AlkaidSolver::default();
    /// let solution = solver.solve_variant::<Sdvrp>(&mut config, &instance);
    /// ```
    pub fn solve_variant<V: VrpVariant>(
        &self,
        config: &mut AlkaidConfig,
        instance: &V::Instance,
    ) -> AlkaidSolution {
        let base = V::base_instance(instance);

        if let Some(ref mut listener) = config.listener {
            listener.on_start();
        }

        let mut random = Random::new(config.random_seed);
        let mut context = RouteContext::new();
        let mut cache_map = CacheMap::new();
        let mut best_solution = AlkaidSolution::new();
        let mut best_objective = i32::MAX;

        let start_time = Instant::now();
        let max_stagnation = config
            .max_stagnation
            .min((base.num_customers as i32) * (V::calc_fleet_lower_bound(instance) as i32));

        while start_time.elapsed().as_secs_f64() < config.time_limit {
            // Construct initial solution (variant-specific)
            let mut solution = V::construct(instance, &mut random);
            let mut objective = V::calc_objective(instance, &solution);
            let mut iter_best_objective = objective;
            let mut new_solution = solution.clone();
            let mut num_stagnation = 0;

            while num_stagnation < max_stagnation
                && start_time.elapsed().as_secs_f64() < config.time_limit
            {
                num_stagnation += 1;

                // Intra-route search on all routes
                context.calc_route_context(&new_solution);
                for i in 0..context.num_routes() {
                    Self::intra_route_search::<V>(
                        instance,
                        config,
                        i,
                        &mut new_solution,
                        &mut context,
                        &mut random,
                    );
                }

                // Inter-route search
                Self::randomized_variable_neighborhood_descent::<V>(
                    instance,
                    config,
                    &mut new_solution,
                    &mut context,
                    &mut random,
                    &mut cache_map,
                );

                let new_objective = V::calc_objective(instance, &new_solution);

                // Update iteration best
                if new_objective < iter_best_objective {
                    num_stagnation = 0;
                    iter_best_objective = new_objective;
                }

                // Update global best
                if new_objective < best_objective {
                    best_objective = new_objective;
                    best_solution = new_solution.clone();
                    if let Some(ref mut listener) = config.listener {
                        listener.on_updated(&best_solution, best_objective);
                    }
                }

                // Accept or reject
                if config
                    .acceptance_rule
                    .accept(objective, new_objective, &mut random)
                {
                    objective = new_objective;
                    solution = new_solution.clone();
                } else {
                    new_solution = solution.clone();
                }

                // Perturb
                Self::perturb::<V>(
                    instance,
                    &config.sorter,
                    config.blink_rate,
                    &mut new_solution,
                    &mut context,
                    &mut random,
                    config.ruin_method.as_mut(),
                );
            }
        }

        if let Some(ref mut listener) = config.listener {
            listener.on_end(&best_solution, best_objective);
        }

        best_solution
    }

    /// Solves an SDVRP instance (backward-compatible convenience method).
    ///
    /// This is equivalent to calling `solve_variant::<Sdvrp>(config, instance)`.
    ///
    /// # Arguments
    ///
    /// * `config` - Solver configuration
    /// * `instance` - The SDVRP problem instance
    ///
    /// # Returns
    ///
    /// The best solution found within the time limit
    pub fn solve(&self, config: &mut AlkaidConfig, instance: &Instance) -> AlkaidSolution {
        self.solve_variant::<Sdvrp>(config, instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acceptance_rule::HillClimbing;
    use crate::inter_operator::SwapStar;
    use crate::intra_operator::Exchange;
    use crate::ruin_method::RandomRuin;
    use crate::sorter::SortByRandom;

    #[test]
    fn test_solver_basic() {
        let instance = Instance {
            num_customers: 3,
            capacity: 100,
            demands: vec![0, 50, 30],
            distance_matrix: vec![vec![0, 10, 20], vec![10, 0, 15], vec![20, 15, 0]],
        };

        let mut sorter = Sorter::new();
        sorter.add_sort_function(Box::new(SortByRandom), 1.0);

        let mut config = AlkaidConfig {
            random_seed: 42,
            max_stagnation: 5000,
            time_limit: 0.1, // Short time limit for test
            blink_rate: 0.01,
            inter_operators: vec![Box::new(SwapStar)],
            intra_operators: vec![Box::new(Exchange)],
            acceptance_rule: Box::new(HillClimbing),
            ruin_method: Box::new(RandomRuin::new(vec![1])),
            sorter,
            listener: None,
        };

        let solver = AlkaidSolver::default();
        let solution = solver.solve(&mut config, &instance);

        // Should have found a valid solution
        assert!(!solution.node_indices().is_empty());
        assert!(solution.calc_objective(&instance) > 0);
    }
}
