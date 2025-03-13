use crate::prover::get_air;
use crate::{MAX_LOG_BLOWUP, TRACES_PATH};
use p3_bls12_377_fr::Bls12_377Fr;
use trace::global::RawGlobalTrace;
use trace::lookup::RawLookupTrace;
use trace::permutation::RawPermutationTrace;
use trace::range::RawRangeTrace;
use trace::{RawProcessedTrace, RawTrace};

pub fn read_range_traces(
    challenges: Vec<Bls12_377Fr>,
    start: usize,
    stop: usize,
) -> Vec<Vec<RawRangeTrace>> {
    let mut range_traces: Vec<Vec<RawRangeTrace>> = vec![vec![]; MAX_LOG_BLOWUP];

    for i in start..stop {
        if let Ok(mut trace) = RawRangeTrace::read_file(&format!("{}/range_{}.bin", TRACES_PATH, i))
        {
            println!(
                "Reading range_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );

            let log_blowup = get_log_blowup(trace.clone(), challenges.clone());
            range_traces[log_blowup].push(trace);
        }
    }

    range_traces
}

pub fn read_lookup_traces(
    challenges: Vec<Bls12_377Fr>,
    start: usize,
    stop: usize,
) -> Vec<Vec<RawLookupTrace>> {
    let mut lookup_traces: Vec<Vec<RawLookupTrace>> = vec![vec![]; MAX_LOG_BLOWUP];

    for i in start..stop {
        if let Ok(trace) = RawLookupTrace::read_file(&format!("{}/lookup_{}.bin", TRACES_PATH, i)) {
            println!(
                "Reading lookup_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );

            let log_blowup = get_log_blowup(trace.clone(), challenges.clone());
            lookup_traces[log_blowup].push(trace);
        }
    }

    lookup_traces
}

pub fn read_global_traces(
    challenges: Vec<Bls12_377Fr>,
    start: usize,
    stop: usize,
) -> Vec<Vec<RawGlobalTrace>> {
    let mut global_traces: Vec<Vec<RawGlobalTrace>> = vec![vec![]; MAX_LOG_BLOWUP];

    for i in start..stop {
        if let Ok(trace) = RawGlobalTrace::read_file(&format!("{}/global{}.bin", TRACES_PATH, i)) {
            println!(
                "Reading global{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );

            let log_blowup = get_log_blowup(trace.clone(), challenges.clone());
            global_traces[log_blowup].push(trace)
        }
    }

    global_traces
}

pub fn read_permutation_traces(
    challenges: Vec<Bls12_377Fr>,
    start: usize,
    stop: usize,
) -> Vec<Vec<RawPermutationTrace>> {
    let mut permutation_traces: Vec<Vec<RawPermutationTrace>> = vec![vec![]; MAX_LOG_BLOWUP];

    for i in start..stop {
        if let Ok(trace) =
            RawPermutationTrace::read_file(&format!("{}/permutation_{}.bin", TRACES_PATH, i))
        {
            println!(
                "Reading permutation_{}.bin -> {}",
                i,
                trace.get_max_height().ilog2() as usize
            );

            let log_blowup = get_log_blowup(trace.clone(), challenges.clone());
            permutation_traces[log_blowup].push(trace)
        }
    }

    permutation_traces
}

fn get_log_blowup<T: RawTrace>(mut trace: T, challenges: Vec<Bls12_377Fr>) -> usize {
    let mut raw_trace = RawProcessedTrace::new(challenges.clone(), trace.get_max_height());

    let cfg = trace.update_processed_trace(&mut raw_trace);

    let matrix = raw_trace.get_trace();

    let (air, _, _) = get_air(&[cfg], &matrix, challenges.clone(), 0);

    trace.get_min_blowup(air, challenges.len())
}
