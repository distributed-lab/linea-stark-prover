mod air_permutation;
mod config;
mod trace;
mod air_lookup;
mod air;

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

fn main() -> Result<(), impl Debug> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    let lookup0 = read_lookup("trace/lookup_0_0.bin");
    let lookup1 = read_lookup("trace/lookup_1_0.bin");
    let lookup2 = read_lookup("trace/lookup_2_0.bin");

    let mut raw_trace = RawTrace::default();
    raw_trace.push_lookup(lookup0);
    raw_trace.push_lookup(lookup1);
    // raw_trace.push_lookup(lookup2);

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
