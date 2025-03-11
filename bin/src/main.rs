mod config;
mod prover;

use crate::prover::{get_air, prove_linea};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use trace::global::RawGlobalTrace;
use trace::range::RawRangeTrace;
use trace::{lookup::RawLookupTrace, permutation::RawPermutationTrace, RawTrace};
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
    let challenges = vec![alpha_challenge, delta_challenge];
    println!("Challenge delta: {}", delta_challenge);
    println!("Challenge alpha: {}", alpha_challenge);

    // read all traces

    let mut lookup_traces: Vec<Vec<RawLookupTrace>> = vec![vec![]; 10];
    let mut permutation_traces: Vec<Vec<RawPermutationTrace>> = vec![vec![]; 10];
    let mut range_traces: Vec<Vec<RawRangeTrace>> = vec![vec![]; 10];
    let mut global_traces: Vec<Vec<(RawGlobalTrace, i32)>> = vec![vec![]; 10];

    let ranges = vec![(0, 68700)];

    for range in ranges {
        let mut read_counter = 0;

        println!("Reading globals in range from {} to {}", range.0, range.1);
        for i in range.0..range.1 {
            if let Ok(trace) = RawGlobalTrace::read_file(&format!(
                "../traces/trace/global{}.bin",
                i
            )) {
                read_counter += 1;

                let mut raw_trace = RawTrace::new(challenges.clone(), trace.get_max_height());

                let cfgs = raw_trace.push_traces(vec![], vec![], vec![], vec![trace.clone()]);

                let matrix = raw_trace.get_trace();

                let (air, _, _) = get_air(&cfgs, &matrix, challenges.clone(), 0);
                global_traces[trace.get_blowup(air, challenges.len())].push((trace, i))
            }
            println!("Read {} from {}", i, range.1);
        }

        println!(
            "Read {}/{} global constraints",
            read_counter,
            range.1 - range.0
        );
    }

    for log_blowup in (1..10).rev() {
        let permutation_trace = permutation_traces.pop().unwrap();
        let lookup_trace = lookup_traces.pop().unwrap();
        let range_trace = range_traces.pop().unwrap();
        let global_trace = global_traces.pop().unwrap();

        println!("Proving log_blowup {}", log_blowup);
        if !global_trace.is_empty() {
            for (i, (trace, file_ind)) in global_trace.iter().enumerate() {
                println!("Proving trace {}/{}. File: global{}.bin. Trace height: {}, expression height: {}, expression_width {}. Log_blowup: {}", i, global_trace.len(), file_ind, trace.get_max_height(), trace.get_expression_height(), trace.get_expression_width(), log_blowup);

                prove_linea(
                    vec![alpha_challenge, delta_challenge],
                    permutation_trace.clone(),
                    lookup_trace.clone(),
                    range_trace.clone(),
                    vec![trace.clone()],
                    trace.get_max_height(),
                    log_blowup,
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
