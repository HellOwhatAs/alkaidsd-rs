//! Benchmark runner for comparing Rust implementation with C++
//!
//! This binary provides a CLI interface compatible with the comparison script.

use alkaidsd::acceptance_rule::{
    AcceptanceRule, HillClimbing, HillClimbingWithEqual, LateAcceptanceHillClimbing,
    SimulatedAnnealing,
};
use alkaidsd::distance_matrix_optimizer::DistanceMatrixOptimizer;
use alkaidsd::inter_operator::{
    Cross, InterOperator, Relocate, SdSwapOneOne, SdSwapStar, SdSwapTwoOne, Swap, SwapStar,
};
use alkaidsd::intra_operator::{Exchange, IntraOperator, OrOpt};
use alkaidsd::ruin_method::{RandomRuin, RuinMethod, SisrsRuin};
use alkaidsd::sorter::{SortByClose, SortByDemand, SortByFar, SortByRandom, Sorter};
use alkaidsd::{AlkaidConfig, AlkaidSolution, AlkaidSolver, Instance, Listener, Node};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;

/// Listener that prints updates to stdout
struct SimpleListener {
    start_time: Instant,
}

impl SimpleListener {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

impl Listener for SimpleListener {
    fn on_start(&mut self) {
        self.start_time = Instant::now();
    }

    fn on_updated(&mut self, _solution: &AlkaidSolution, objective: i32) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        println!("Update at {:.3}s: {}", elapsed, objective);
    }

    fn on_end(&mut self, _solution: &AlkaidSolution, objective: i32) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        println!("End at {:.3}s: {}", elapsed, objective);
    }
}

/// Parse an instance file in the COORD_LIST format
fn read_instance(path: &Path) -> Result<Instance, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Read first line: num_customers capacity
    let first_line = lines
        .next()
        .ok_or("Empty file")?
        .map_err(|e| format!("Read error: {}", e))?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid format: first line should contain num_customers and capacity".into());
    }

    let num_customers: Node = parts[0].parse().map_err(|_| "Invalid num_customers")?;
    let num_customers = num_customers + 1; // Add depot
    let capacity: i32 = parts[1].parse().map_err(|_| "Invalid capacity")?;

    // Read demands line
    let demands_line = lines
        .next()
        .ok_or("Missing demands line")?
        .map_err(|e| format!("Read error: {}", e))?;
    let mut demands: Vec<i32> = vec![0]; // Depot has 0 demand
    for part in demands_line.split_whitespace() {
        if let Ok(d) = part.parse::<i32>() {
            demands.push(d);
        }
    }

    // Ensure we have enough demands
    while demands.len() < num_customers as usize {
        demands.push(0);
    }

    // Read coordinates
    let mut coords: Vec<(i32, i32)> = Vec::new();
    for line in lines {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let x: i32 = parts[0].parse().unwrap_or(0);
            let y: i32 = parts[1].parse().unwrap_or(0);
            coords.push((x, y));
        }
        if coords.len() >= num_customers as usize {
            break;
        }
    }

    // Ensure we have enough coordinates
    while coords.len() < num_customers as usize {
        coords.push((0, 0));
    }

    // Build distance matrix
    let mut distance_matrix: Vec<Vec<i32>> =
        vec![vec![0; num_customers as usize]; num_customers as usize];
    for i in 0..num_customers as usize {
        for j in 0..num_customers as usize {
            let (x1, y1) = coords[i];
            let (x2, y2) = coords[j];
            let dx = (x1 - x2) as f64;
            let dy = (y1 - y2) as f64;
            distance_matrix[i][j] = (dx * dx + dy * dy).sqrt().round() as i32;
        }
    }

    Ok(Instance::new(
        num_customers,
        capacity,
        demands,
        distance_matrix,
    ))
}

/// Parse command-line arguments
#[derive(Default)]
struct Args {
    input: String,
    output: String,
    random_seed: u32,
    time_limit: f64,
    blink_rate: f64,
    inter_operators: Vec<String>,
    intra_operators: Vec<String>,
    acceptance_rule_type: String,
    acceptance_rule_args: Vec<String>,
    ruin_method_type: String,
    ruin_method_args: Vec<String>,
    sorters: Vec<String>,
}

fn parse_string_list(s: &str) -> Vec<String> {
    // Parse JSON-like list: ["item1", "item2", ...]
    // Handles items with commas inside angle brackets like "Swap<2, 0>"
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        let mut result = Vec::new();
        let mut current = String::new();
        let mut angle_depth = 0;
        let mut in_quotes = false;

        for ch in inner.chars() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                '<' if !in_quotes => {
                    angle_depth += 1;
                    current.push(ch);
                }
                '>' if !in_quotes => {
                    angle_depth -= 1;
                    current.push(ch);
                }
                ',' if !in_quotes && angle_depth == 0 => {
                    let item = current.trim().trim_matches('"').to_string();
                    if !item.is_empty() {
                        result.push(item);
                    }
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // Don't forget the last item
        let item = current.trim().trim_matches('"').to_string();
        if !item.is_empty() {
            result.push(item);
        }

        result
    } else {
        vec![s.to_string()]
    }
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().collect();
    let mut result = Args::default();

    // Default values
    result.random_seed = 42;
    result.time_limit = 10.0;
    result.blink_rate = 0.021;
    result.acceptance_rule_type = "LAHC".to_string();
    result.ruin_method_type = "SISRs".to_string();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        // Handle --config=file or --config file
        if arg.starts_with("--config=") {
            let config_path = &arg[9..];
            parse_config_file(config_path, &mut result)?;
        } else if arg == "--config" && i + 1 < args.len() {
            i += 1;
            parse_config_file(&args[i], &mut result)?;
        } else if arg.starts_with("--input=") {
            result.input = arg[8..].to_string();
        } else if arg == "--input" && i + 1 < args.len() {
            i += 1;
            result.input = args[i].clone();
        } else if arg.starts_with("--output=") {
            result.output = arg[9..].to_string();
        } else if arg == "--output" && i + 1 < args.len() {
            i += 1;
            result.output = args[i].clone();
        } else if arg.starts_with("--random-seed=") {
            result.random_seed = arg[14..].parse().map_err(|_| "Invalid random-seed")?;
        } else if arg == "--random-seed" && i + 1 < args.len() {
            i += 1;
            result.random_seed = args[i].parse().map_err(|_| "Invalid random-seed")?;
        } else if arg.starts_with("--time-limit=") {
            result.time_limit = arg[13..].parse().map_err(|_| "Invalid time-limit")?;
        } else if arg == "--time-limit" && i + 1 < args.len() {
            i += 1;
            result.time_limit = args[i].parse().map_err(|_| "Invalid time-limit")?;
        } else if arg.starts_with("--blink-rate=") {
            result.blink_rate = arg[13..].parse().map_err(|_| "Invalid blink-rate")?;
        } else if arg == "--blink-rate" && i + 1 < args.len() {
            i += 1;
            result.blink_rate = args[i].parse().map_err(|_| "Invalid blink-rate")?;
        }

        i += 1;
    }

    Ok(result)
}

fn parse_config_file(path: &str, args: &mut Args) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Cannot open config file: {}", e))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        // Parse key = value
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();

            match key {
                "input" => args.input = value.to_string(),
                "output" => args.output = value.to_string(),
                "random-seed" => args.random_seed = value.parse().unwrap_or(42),
                "time-limit" => args.time_limit = value.parse().unwrap_or(10.0),
                "blink-rate" => args.blink_rate = value.parse().unwrap_or(0.021),
                "inter-operators" => args.inter_operators = parse_string_list(value),
                "intra-operators" => args.intra_operators = parse_string_list(value),
                "acceptance-rule-type" => {
                    args.acceptance_rule_type = value.trim_matches('"').to_string()
                }
                "acceptance-rule-args" => args.acceptance_rule_args = parse_string_list(value),
                "ruin-method-type" => args.ruin_method_type = value.trim_matches('"').to_string(),
                "ruin-method-args" => args.ruin_method_args = parse_string_list(value),
                "sorters" => args.sorters = parse_string_list(value),
                _ => {}
            }
        }
    }

    Ok(())
}

fn parse_key_value(s: &str) -> Option<(String, f64)> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() == 2 {
        if let Ok(value) = parts[1].parse::<f64>() {
            return Some((parts[0].to_string(), value));
        }
    }
    None
}

fn build_inter_operators(names: &[String]) -> Vec<Box<dyn InterOperator>> {
    let mut ops: Vec<Box<dyn InterOperator>> = Vec::new();

    for name in names {
        match name.as_str() {
            "Relocate" => ops.push(Box::new(Relocate)),
            "Swap<2, 0>" | "Swap<2,0>" => ops.push(Box::new(Swap::<2, 0>::default())),
            "Swap<2, 1>" | "Swap<2,1>" => ops.push(Box::new(Swap::<2, 1>::default())),
            "Swap<2, 2>" | "Swap<2,2>" => ops.push(Box::new(Swap::<2, 2>::default())),
            "Cross" => ops.push(Box::new(Cross)),
            "SwapStar" => ops.push(Box::new(SwapStar)),
            "SdSwapStar" => ops.push(Box::new(SdSwapStar)),
            "SdSwapOneOne" => ops.push(Box::new(SdSwapOneOne)),
            "SdSwapTwoOne" => ops.push(Box::new(SdSwapTwoOne)),
            _ => eprintln!("Unknown inter-operator: {}", name),
        }
    }

    // Default operators if none specified
    if ops.is_empty() {
        ops.push(Box::new(Relocate));
        ops.push(Box::new(Swap::<2, 0>::default()));
        ops.push(Box::new(Swap::<2, 1>::default()));
        ops.push(Box::new(Swap::<2, 2>::default()));
        ops.push(Box::new(Cross));
        ops.push(Box::new(SwapStar));
        ops.push(Box::new(SdSwapStar));
    }

    ops
}

fn build_intra_operators(names: &[String]) -> Vec<Box<dyn IntraOperator>> {
    let mut ops: Vec<Box<dyn IntraOperator>> = Vec::new();

    for name in names {
        match name.as_str() {
            "Exchange" => ops.push(Box::new(Exchange)),
            "OrOpt<1>" => ops.push(Box::new(OrOpt::<1>::default())),
            "OrOpt<2>" => ops.push(Box::new(OrOpt::<2>::default())),
            "OrOpt<3>" => ops.push(Box::new(OrOpt::<3>::default())),
            _ => eprintln!("Unknown intra-operator: {}", name),
        }
    }

    // Default operators if none specified
    if ops.is_empty() {
        ops.push(Box::new(Exchange));
        ops.push(Box::new(OrOpt::<1>::default()));
    }

    ops
}

fn build_acceptance_rule(
    rule_type: &str,
    args: &[String],
) -> Box<dyn Fn() -> Box<dyn AcceptanceRule>> {
    let mut params: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for arg in args {
        if let Some((k, v)) = parse_key_value(arg) {
            params.insert(k, v);
        }
    }

    match rule_type {
        "LAHC" => {
            let length = params.get("length").copied().unwrap_or(83.0) as usize;
            Box::new(move || Box::new(LateAcceptanceHillClimbing::new(length)))
        }
        "SA" => {
            let initial_temp = params.get("initial_temperature").copied().unwrap_or(1000.0);
            let decay = params.get("decay").copied().unwrap_or(0.99);
            Box::new(move || Box::new(SimulatedAnnealing::new(initial_temp, decay)))
        }
        "HCWE" => Box::new(|| Box::new(HillClimbingWithEqual)),
        _ => Box::new(|| Box::new(HillClimbing)),
    }
}

fn build_ruin_method(method_type: &str, args: &[String]) -> Box<dyn RuinMethod> {
    let mut params: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for arg in args {
        if let Some((k, v)) = parse_key_value(arg) {
            params.insert(k, v);
        }
    }

    match method_type {
        "SISRs" => {
            let avg_customers = params.get("average_customers").copied().unwrap_or(36.0) as i32;
            let max_length = params.get("max_length").copied().unwrap_or(8.0) as i32;
            let split_rate = params.get("split_rate").copied().unwrap_or(0.74);
            let preserved_prob = params
                .get("preserved_probability")
                .copied()
                .unwrap_or(0.096);
            Box::new(SisrsRuin::new(
                avg_customers,
                max_length,
                split_rate,
                preserved_prob,
            ))
        }
        "Random" => {
            let sizes: Vec<i32> = args.iter().filter_map(|s| s.parse().ok()).collect();
            if sizes.is_empty() {
                Box::new(RandomRuin::new(vec![3, 5, 7]))
            } else {
                Box::new(RandomRuin::new(sizes))
            }
        }
        _ => Box::new(SisrsRuin::new(36, 8, 0.74, 0.096)),
    }
}

fn build_sorter(sorter_args: &[String]) -> Sorter {
    let mut sorter = Sorter::new();

    // Parse key-value pairs and sort by key name to match C++ std::map ordering
    let mut parsed: Vec<(String, f64)> = Vec::new();
    for arg in sorter_args {
        if let Some((name, weight)) = parse_key_value(arg) {
            parsed.push((name, weight));
        }
    }
    parsed.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, weight) in &parsed {
        match name.as_str() {
            "random" => sorter.add_sort_function(Box::new(SortByRandom), *weight),
            "demand" => sorter.add_sort_function(Box::new(SortByDemand), *weight),
            "far" => sorter.add_sort_function(Box::new(SortByFar), *weight),
            "close" => sorter.add_sort_function(Box::new(SortByClose), *weight),
            _ => eprintln!("Unknown sorter: {}", name),
        }
    }

    // Default sorter if none specified
    if sorter_args.is_empty() {
        sorter.add_sort_function(Box::new(SortByClose), 0.120);
        sorter.add_sort_function(Box::new(SortByDemand), 0.225);
        sorter.add_sort_function(Box::new(SortByFar), 0.942);
        sorter.add_sort_function(Box::new(SortByRandom), 0.078);
    }

    sorter
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    if args.input.is_empty() {
        eprintln!("Error: --input is required");
        std::process::exit(1);
    }

    if args.output.is_empty() {
        eprintln!("Error: --output is required");
        std::process::exit(1);
    }

    // Read instance
    let mut instance = match read_instance(Path::new(&args.input)) {
        Ok(instance) => instance,
        Err(e) => {
            eprintln!("Error reading instance: {}", e);
            std::process::exit(1);
        }
    };

    // Optimize distance matrix using Floyd-Warshall (same as C++ implementation)
    let optimizer = DistanceMatrixOptimizer::new(&mut instance.distance_matrix);

    // Build configuration
    let mut config = AlkaidConfig {
        random_seed: args.random_seed,
        max_stagnation: 5000,
        time_limit: args.time_limit,
        blink_rate: args.blink_rate,
        inter_operators: build_inter_operators(&args.inter_operators),
        intra_operators: build_intra_operators(&args.intra_operators),
        acceptance_rule: build_acceptance_rule(
            &args.acceptance_rule_type,
            &args.acceptance_rule_args,
        ),
        ruin_method: build_ruin_method(&args.ruin_method_type, &args.ruin_method_args),
        sorter: build_sorter(&args.sorters),
        listener: Some(Box::new(SimpleListener::new())),
    };

    // Solve
    let solver = AlkaidSolver::default();
    let solution = solver.solve(&mut config, &instance);

    // Restore intermediate nodes from Floyd-Warshall optimization
    let mut solution = solution;
    optimizer.restore(&mut solution);

    // Write output
    if let Ok(mut file) = File::create(&args.output) {
        let _ = writeln!(file, "Objective: {}", solution.calc_objective(&instance));
        // Additional solution details could be written here
    }
}
