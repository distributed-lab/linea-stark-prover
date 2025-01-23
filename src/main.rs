mod air_permutation;
mod config;
mod trace;
mod air_lookup;
mod air;

use std::collections::HashSet;
use crate::air_permutation::LineaPermutationAIR;
use crate::config::*;
use crate::trace::{RawTrace, read_lookup};
use ark_ff::PrimeField;
use p3_field::{Field, FieldAlgebra};
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_matrix::Matrix;
use p3_uni_stark::{prove, verify};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::fmt::Debug;
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use crate::air::{AirConfig, LineaAIR};
use crate::air_lookup::LineaLookupAIR;

fn dummy_check<F: Field + Ord>(mut a: Vec<Vec<F>>, mut b: Vec<Vec<F>>) {
    let mut a_all: Vec<F> = Vec::new();
    let mut b_all: Vec<F> = Vec::new();

    for i in 0..a.len() {
        a_all.append(&mut a[i]);
        b_all.append(&mut b[i]);
    }

    a_all.sort();
    b_all.sort();

    assert_eq!(a_all.len(), b_all.len());
    for i in 0..a_all.len() {
        assert_eq!(a_all[i], b_all[i]);
    }
}

/// Returns true if the check is passed. Otherwise, returns true.
fn dummy_lookup_check<F: Field + Ord>(mut a: Vec<Vec<F>>, mut b: Vec<Vec<F>>) -> bool {
    let mut b_all = HashSet::new();

    for i in 0..b.len() {
        for e in &b[i] {
            b_all.insert(e);
        }
    }

    for i in 0..a.len() {
        for e in &a[i] {
            if b_all.get(e).is_none() {
                return false
            }
        }
    }

    return true
}

fn main() -> Result<(), impl Debug> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    let mut raw_trace = RawTrace::default();

    let skip_indexes = vec![8, 11, 12, 13, 15, 19, 22, 33, 36, 39, 40];
    for i in 8..=8 {
        // if skip_indexes.contains(&i) {
        //     continue
        // }

        let name = format!("trace/lookup_{}_0.bin", i);
        println!("reading {}", name);

        let lookup = read_lookup(&name);
        raw_trace.push_lookup(lookup.clone());

        println!("max height: {}", raw_trace.max_height);
    }

    // let mut failed_files = vec![];
    // for i in 0..=40 {
    //     let name = format!("trace/lookup_{}_0.bin", i);
    //
    //     let lookup = read_lookup(&name);
    //     let (a, b) = lookup.get_columns();
    //
    //     if dummy_lookup_check(a, b) {
    //         println!("Passed: {}", name);
    //     } else {
    //         println!("FAILED: {} <----------------- Failed file", name);
    //         failed_files.push(name)
    //     }
    // }

    // println!("Failed {} files", failed_files.len());
    // println!("Failed files names: {:?}", failed_files);

    // dummy_check(a, b);

    // -----------------------------------------------------------

    // TODO: should not be just random
    let mut rng = thread_rng();
    let challenge = rng.sample(Standard {});
    println!("Challenge: {}", challenge);

    let perm = Perm::new_from_rng(8, 22, &mut rng);
    let hash = Hash::new(perm.clone());

    let dft = Dft::default();

    // TODO: use proper PCS configured with FRI config
    let compress = Compress::new(hash.clone());
    let val_mmcs = ValMmcs::new(hash.clone(), compress.clone());
    let challenge_mmcs = ChallengeMmcs::new(hash.clone(), compress.clone());
    let fri_config = FriConfig {
        log_blowup: 2,
        log_final_poly_len: 0,
        num_queries: 33,
        proof_of_work_bits: 0, //29
        mmcs: challenge_mmcs,
    };

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    let config = Config::new(pcs);

    println!("Generating trace...");
    let trace = raw_trace.get_trace(challenge);
    println!("width: {}", trace.width());

    println!("Creating LineaAir...");
    // let air = LineaLookupAIR::new(4, 4, challenge);
    let air = LineaAIR::new(raw_trace.get_air_configs(), challenge);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Proving...");
    let proof = prove(&config, &air, &mut challenger, trace, &vec![]);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Verification...");
    verify(&config, &air, &mut challenger, &proof, &vec![])
}



fn a() -> Result<(), impl Debug> {
// fn main() -> Result<(), impl Debug> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    let trace = read_lookup("trace/lookup_2_0.bin");

    // dummy_check(a, b);

    // -----------------------------------------------------------

    // TODO: should not be just random
    let mut rng = thread_rng();
    let challenge = rng.sample(Standard {});
    println!("Challenge: {}", challenge);

    let perm = Perm::new_from_rng(8, 22, &mut rng);
    let hash = Hash::new(perm.clone());

    let dft = Dft::default();

    // TODO: use proper PCS configured with FRI config
    let compress = Compress::new(hash.clone());
    let val_mmcs = ValMmcs::new(hash.clone(), compress.clone());
    let challenge_mmcs = ChallengeMmcs::new(hash.clone(), compress.clone());
    let fri_config = FriConfig {
        log_blowup: 2,
        log_final_poly_len: 0,
        num_queries: 33,
        proof_of_work_bits: 0,
        mmcs: challenge_mmcs,
    };

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    let config = Config::new(pcs);

    println!("Generating trace...");
    let a_width = trace.a_width();
    let b_width = trace.b_width();
    let trace = trace.get_trace(challenge);

    let air = LineaLookupAIR::new(a_width, b_width, challenge);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Proving...");
    let proof = prove(&config, &air, &mut challenger, trace, &vec![]);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Verification...");
    verify(&config, &air, &mut challenger, &proof, &vec![])
}
