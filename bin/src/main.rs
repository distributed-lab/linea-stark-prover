mod config;

use crate::config::*;
use air::LineaAIR;
use p3_field::Field;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_uni_stark::{prove, verify};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::fmt::Debug;
use trace::{lookup::RawLookupTrace, permutation::RawPermutationTrace, RawTrace};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

fn main() -> Result<(), impl Debug> {
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

    let mut raw_trace = RawTrace::new(vec![alpha_challenge, delta_challenge]);

    let lookup_traces = vec![
        RawLookupTrace::read_file("../lookup_0.bin"),
        RawLookupTrace::read_file("../lookup_0.bin"),
        RawLookupTrace::read_file("../lookup_0.bin"),
        RawLookupTrace::read_file("../lookup_0.bin"),
        RawLookupTrace::read_file("../lookup_0.bin"),
    ];

    let permutation_traces = vec![
        RawPermutationTrace::read_file("../permutation_0.bin"),
        RawPermutationTrace::read_file("../permutation_0.bin"),
        RawPermutationTrace::read_file("../permutation_0.bin"),
        RawPermutationTrace::read_file("../permutation_0.bin"),
        RawPermutationTrace::read_file("../permutation_0.bin"),
    ];

    let cfgs = raw_trace.push_traces(permutation_traces, lookup_traces);

    // -----------------------------------------------------------

    // TODO: should not be just random

    let perm = Perm::new_from_rng(8, 22, &mut rng);
    let hash = Hash::new(perm.clone());

    let dft = Dft::default();

    // TODO: use proper PCS configured with FRI config
    let compress = Compress::new(hash.clone());
    let val_mmcs = ValMmcs::new(hash.clone(), compress.clone());
    let challenge_mmcs = ChallengeMmcs::new(hash.clone(), compress.clone());
    let fri_config = FriConfig {
        log_blowup: 3,
        log_final_poly_len: 0,
        num_queries: 33,
        proof_of_work_bits: 0, //29
        mmcs: challenge_mmcs,
    };

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    let config = Config::new(pcs);

    println!("Generating trace...");

    let t = raw_trace.get_trace();

    println!("Creating LineaAir...");

    let air = LineaAIR::new(cfgs);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Proving...");
    let proof = prove(
        &config,
        &air,
        &mut challenger,
        t,
        &vec![alpha_challenge, delta_challenge],
    );

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Verification...");
    verify(
        &config,
        &air,
        &mut challenger,
        &proof,
        &vec![alpha_challenge, delta_challenge],
    )
}
