mod config;
mod prover;

use crate::prover::prove_linea;
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use trace::global::RawGlobalTrace;
use trace::range::RawRangeTrace;
use trace::{lookup::RawLookupTrace, permutation::RawPermutationTrace};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    let mut rng = thread_rng();
    let alpha_challenge = rng.sample(Standard {});
    let delta_challenge = rng.sample(Standard {});
    println!("Challenge delta: {}", delta_challenge);
    println!("Challenge alpha: {}", alpha_challenge);

    // read all traces

    let mut lookup_traces: Vec<Vec<RawLookupTrace>> = vec![vec![]; 70];
    let mut permutation_traces: Vec<Vec<RawPermutationTrace>> = vec![vec![]; 70];
    let mut range_traces: Vec<Vec<RawRangeTrace>> = vec![vec![]; 70];
    let mut global_traces: Vec<Vec<RawGlobalTrace>> = vec![vec![]; 70];

    let ranges = vec![
        (0, 68700)
    ];

    for range in ranges {
        let mut read_counter = 0;

        println!("Reading globals in range from {} to {}", range.0, range.1);
        for i in range.0..range.1 {
            if let Ok(trace) = RawGlobalTrace::read_file(&format!("../global{}.bin", i)) {
                read_counter += 1;

                global_traces[trace.get_expression_height()].push(trace)
            }
            println!("Read {} from {}", i, range.1);
        }

        println!(
            "Read {}/{} global constraints",
            read_counter,
            range.1 - range.0
        );
    }

    for expression_height in (1..70).rev() {
        let permutation_trace = permutation_traces.pop().unwrap();
        let lookup_trace = lookup_traces.pop().unwrap();
        let range_trace = range_traces.pop().unwrap();
        let global_trace = global_traces.pop().unwrap();

        println!("Proving expression height {}", expression_height);
        if !global_trace.is_empty() {
            for (i, trace) in global_trace.iter().enumerate() {
                println!("Proving trace {}/{}", i, global_trace.len());

                prove_linea(
                    vec![alpha_challenge, delta_challenge],
                    permutation_trace.clone(),
                    lookup_trace.clone(),
                    range_trace.clone(),
                    vec![trace.clone()],
                    trace.get_max_height(),
                );
            }
        }
    }
}

fn group_by_expression_height(globals: Vec<RawGlobalTrace>) -> HashMap<usize, Vec<RawGlobalTrace>> {
    let mut res: HashMap<usize, Vec<RawGlobalTrace>> = HashMap::new();

    for global in globals {
        let expression_height = global.get_expression_height();

        if let Some(val) = res.get_mut(&global.get_expression_height()) {
            val.push(global.clone());
        } else {
            res.insert(expression_height, vec![global.clone()]);
        }
    }

    res
}
