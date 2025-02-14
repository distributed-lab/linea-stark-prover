mod config;
mod prover;

use crate::prover::prove_linea;
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use trace::{lookup::RawLookupTrace, permutation::RawPermutationTrace};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use trace::range::RawRangeTrace;

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

    for i in 0..0 {
        if let Ok(trace) = RawRangeTrace::read_file(&format!("../range_{}.bin", i)) {
            println!(
                "Reading range_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );
            range_traces[trace.get_max_height().ilog2() as usize].push(trace);
        }

    }

    for i in 18..19 {
        if let Ok(trace) = RawLookupTrace::read_file(&format!("../lookup_{}.bin", i)) {
            println!(
                "Reading lookup_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );
            lookup_traces[trace.get_max_height().ilog2() as usize].push(trace.clone());
        }
    }

    for i in 0..0 {
        if let Ok(trace) =
            RawPermutationTrace::read_file(&format!("../permutation_{}.bin", i))
        {
            println!(
                "Reading permutation_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );
            permutation_traces[trace.get_max_height().ilog2() as usize].push(trace);
        }
    }

    let mut height = 1 << (permutation_traces.len() - 1);

    for i in 0..31 {

        let permutation_trace = permutation_traces.pop().unwrap();
        let lookup_trace = lookup_traces.pop().unwrap();
        let range_trace = range_traces.pop().unwrap();

        if !permutation_trace.is_empty() || !lookup_trace.is_empty() || !range_trace.is_empty() {
            println!(
                "Proving for height 2^{}: {}x lookups, {}x perms, {}x ranges.",
                31 - i,
                lookup_trace.len(),
                permutation_trace.len(),
                range_trace.len(),
            );
            prove_linea(
                vec![alpha_challenge, delta_challenge],
                permutation_trace,
                lookup_trace,
                range_trace,
                height,
            );
        }

        height >>= 1;
    }
}
