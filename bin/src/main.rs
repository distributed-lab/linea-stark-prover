mod config;
mod prover;

use crate::config::*;
use crate::prover::prove_linea;
use air::LineaAIR;
use p3_field::Field;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_uni_stark::{prove, verify};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::fmt::Debug;
use trace::lookup_no_filter::RawLookupNoFilterTrace;
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
    println!("Challenge delta: {}", delta_challenge);
    println!("Challenge alpha: {}", alpha_challenge);

    // read all traces

    let mut lookup_no_filter_traces: Vec<Vec<RawLookupNoFilterTrace>> = vec![vec![]; 32];
    let mut lookup_traces: Vec<Vec<RawLookupTrace>> = vec![vec![]; 32];
    let mut permutation_traces: Vec<Vec<RawPermutationTrace>> = vec![vec![]; 32];

    for i in 0..973 {
        if let Ok(trace) = RawLookupNoFilterTrace::read_file(&format!(
            "/Users/nazarevsky/Documents/linea/new-lookup-trace/traces/lookup_no_filter_{}.bin",
            i
        )) {
            lookup_no_filter_traces[trace.get_max_height().ilog2() as usize].push(trace);   
        }
    }

    for i in 0..973 {
        if let Ok(trace) = RawLookupTrace::read_file(&format!(
            "../traces/lookup_{}.bin",
            i
        )) {
            lookup_traces[trace.get_max_height().ilog2() as usize].push(trace);
        }
    }
    
    for i in 0..1 {
        if let Ok(trace) = RawPermutationTrace::read_file(&format!(
            "../traces/permutation_{}.bin",
            i
        )) {
            permutation_traces[trace.get_height().ilog2() as usize].push(trace);
        }
    }

    for i in (0..31).rev() {
        let permutation_trace = permutation_traces.pop().unwrap();
        let lookup_trace = lookup_traces.pop().unwrap();
        let lookup_no_filter_trace = lookup_no_filter_traces.pop().unwrap();

        if !permutation_trace.is_empty()
            || !lookup_trace.is_empty()
            || !lookup_no_filter_trace.is_empty()
        {
            println!(
                "Proving for height 2^{}: {}x lookups, {}x lookups with no filters, {}x perms",
                i,
                lookup_trace.len(),
                lookup_no_filter_trace.len(),
                permutation_trace.len()
            );
            prove_linea(
                vec![alpha_challenge, delta_challenge],
                permutation_trace,
                lookup_trace,
                lookup_no_filter_trace,
                1 << (i + 1),
            );
        }
    }
}
