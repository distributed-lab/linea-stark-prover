extern crate core;

mod config;
mod prover;
mod reader;

use crate::prover::prove_linea;
use crate::reader::{
    read_global_traces, read_lookup_traces, read_permutation_traces, read_range_traces,
};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::cmp::max;
use trace::global::RawGlobalTrace;
use trace::range::RawRangeTrace;
use trace::{lookup::RawLookupTrace, permutation::RawPermutationTrace, RawTrace};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

const MAX_LOG_BLOWUP: usize = 15;
const TRACES_PATH: &str = "../traces/trace";

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
    let mut range_traces = read_range_traces(challenges.clone(), 0, 0);
    let mut lookup_traces = read_lookup_traces(challenges.clone(), 0, 0);
    let mut global_traces = read_global_traces(challenges.clone(), 0, 2);
    let mut permutation_traces = read_permutation_traces(challenges, 0, 0);

    for log_blowup in (1..global_traces.len()).rev() {
        println!(
            "Log blowup: {}. Total: {}. Ranges: {}, lookups: {}, globals: {}",
            log_blowup,
            global_traces[log_blowup].len()
                + lookup_traces[log_blowup].len()
                + range_traces[log_blowup].len(),
            range_traces[log_blowup].len(),
            lookup_traces[log_blowup].len(),
            global_traces[log_blowup].len()
        )
    }

    for log_blowup in (1..MAX_LOG_BLOWUP).rev() {
        let permutation_trace = permutation_traces.pop().unwrap();
        let lookup_trace = lookup_traces.pop().unwrap();
        let range_trace = range_traces.pop().unwrap();
        let global_trace = global_traces.pop().unwrap();

        if !permutation_trace.is_empty()
            || !lookup_trace.is_empty()
            || !range_trace.is_empty()
            || !global_trace.is_empty()
        {
            println!("Proving log_blowup {}", log_blowup);

            let height = get_max_height(
                &permutation_trace,
                &lookup_trace,
                &range_trace,
                &global_trace,
            );

            prove_linea(
                vec![alpha_challenge, delta_challenge],
                permutation_trace.clone(),
                lookup_trace.clone(),
                range_trace.clone(),
                global_trace.clone(),
                height,
                log_blowup,
            );
        }
    }
}

fn get_max_height(
    permutation_traces: &[RawPermutationTrace],
    lookup_traces: &[RawLookupTrace],
    range_traces: &[RawRangeTrace],
    global_traces: &[RawGlobalTrace],
) -> usize {
    let mut max_height: usize = 0;

    permutation_traces
        .iter()
        .for_each(|trace| max_height = max(max_height, trace.get_max_height()));
    lookup_traces
        .iter()
        .for_each(|trace| max_height = max(max_height, trace.get_max_height()));
    range_traces
        .iter()
        .for_each(|trace| max_height = max(max_height, trace.get_max_height()));
    global_traces
        .iter()
        .for_each(|trace| max_height = max(max_height, trace.get_max_height()));

    max_height
}
