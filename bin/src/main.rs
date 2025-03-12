mod config;
mod prover;

use crate::prover::{get_air, prove_linea};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::cmp::max;
use std::collections::HashMap;
use trace::global::RawGlobalTrace;
use trace::range::RawRangeTrace;
use trace::{lookup::RawLookupTrace, permutation::RawPermutationTrace, RawTrace};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

const MAX_LOG_BLOWUP: usize = 15;

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

    let mut lookup_traces: Vec<Vec<RawLookupTrace>> = vec![vec![]; MAX_LOG_BLOWUP];
    let mut permutation_traces: Vec<Vec<RawPermutationTrace>> = vec![vec![]; MAX_LOG_BLOWUP];
    let mut range_traces: Vec<Vec<RawRangeTrace>> = vec![vec![]; MAX_LOG_BLOWUP];
    let mut global_traces: Vec<Vec<RawGlobalTrace>> = vec![vec![]; MAX_LOG_BLOWUP];

    for i in 0..1416 {
        if let Ok(trace) = RawRangeTrace::read_file(&format!("../range_{}.bin", i)) {
            println!(
                "Reading range_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );

            let mut raw_trace = RawTrace::new(challenges.clone(), trace.get_max_height());

            let cfgs = raw_trace.push_traces(vec![], vec![], vec![trace.clone()], vec![]);

            let matrix = raw_trace.get_trace();

            let (air, _, _) = get_air(&cfgs, &matrix, challenges.clone(), 0);

            range_traces[trace.get_min_blowup(air, challenges.len())].push(trace);
        }
    }

    for i in 0..973 {
        if let Ok(trace) = RawLookupTrace::read_file(&format!("../lookup_{}.bin", i)) {
            println!(
                "Reading lookup_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );

            let mut raw_trace = RawTrace::new(challenges.clone(), trace.get_max_height());

            let cfgs = raw_trace.push_traces(vec![], vec![trace.clone()], vec![], vec![]);

            let matrix = raw_trace.get_trace();

            let (air, _, _) = get_air(&cfgs, &matrix, challenges.clone(), 0);

            lookup_traces[trace.get_min_blowup(air, challenges.len())].push(trace);
        }
    }

    for i in 0..68700 {
        if let Ok(trace) = RawGlobalTrace::read_file(&format!("../traces/trace/global{}.bin", i)) {
            println!(
                "Reading global{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );

            let mut raw_trace = RawTrace::new(challenges.clone(), trace.get_max_height());

            let cfgs = raw_trace.push_traces(vec![], vec![], vec![], vec![trace.clone()]);

            let matrix = raw_trace.get_trace();

            let (air, _, _) = get_air(&cfgs, &matrix, challenges.clone(), 0);
            global_traces[trace.get_min_blowup(air, challenges.len())].push(trace)
        }
    }

    for log_blowup in 1..global_traces.len() {
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

        println!("Proving log_blowup {}", log_blowup);

        if !permutation_trace.is_empty()
            || !lookup_trace.is_empty()
            || !range_trace.is_empty()
            || !global_trace.is_empty()
        {
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
    permutation_traces: &Vec<RawPermutationTrace>,
    lookup_traces: &Vec<RawLookupTrace>,
    range_traces: &Vec<RawRangeTrace>,
    global_traces: &Vec<RawGlobalTrace>,
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
