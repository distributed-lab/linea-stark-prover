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

    let mut lookup_traces: Vec<Vec<RawLookupTrace>> = vec![vec![]; 32];
    let mut permutation_traces: Vec<Vec<RawPermutationTrace>> = vec![vec![]; 32];
    let mut range_traces: Vec<Vec<RawRangeTrace>> = vec![vec![]; 32];
    let mut global_traces: Vec<Vec<RawGlobalTrace>> = vec![vec![]; 32];

    let ranges = vec![
        (0, 20000),
        (20000, 40000),
        (40000, 60000),
        (60000, 68000)
    ];

    for range in ranges {
        let mut read_counter = 0;

        println!("Reading globals in range from {} to {}", range.0, range.1);
        for i in range.0..range.1 {
            if let Ok(trace) = RawGlobalTrace::read_file(&format!("../traces/trace/global{}.bin", i)) {
                read_counter += 1;
                global_traces[trace.get_max_height().ilog2() as usize].push(trace);
            }
        }

        println!(
            "Read {}/{} global constraints",
            read_counter,
            range.1 - range.0
        );

        let mut height = 1 << (permutation_traces.len() - 1);

        for i in 0..31 {
            let permutation_trace = permutation_traces.pop().unwrap();
            let lookup_trace = lookup_traces.pop().unwrap();
            let range_trace = range_traces.pop().unwrap();
            let global_trace = global_traces.pop().unwrap();

            let traces = group_by_expression_height(global_trace.clone());

            for (expression_height, trace_vec) in traces {
                if !trace_vec.is_empty() {
                    println!(
                        "Proving for height 2^{}. Expression height: {}. {}x lookups, {}x perms, {}x ranges, {}x globals.",
                        31 - i,
                        expression_height,
                        lookup_trace.len(),
                        permutation_trace.len(),
                        range_trace.len(),
                        trace_vec.len(),
                    );
                    prove_linea(
                        vec![alpha_challenge, delta_challenge],
                        permutation_trace.clone(),
                        lookup_trace.clone(),
                        range_trace.clone(),
                        trace_vec,
                        height,
                    );
                }
            }

            height >>= 1;
        }

        lookup_traces = vec![vec![]; 32];
        permutation_traces = vec![vec![]; 32];
        range_traces = vec![vec![]; 32];
        global_traces = vec![vec![]; 32];
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
