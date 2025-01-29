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

fn dummy_permutation_check<F: Field + Ord>(mut a: Vec<Vec<F>>, mut b: Vec<Vec<F>>) {
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
fn dummy_lookup_check<F: Field + Ord>(a: Vec<Vec<F>>, b: Vec<Vec<F>>) -> bool {
    let mut b_all = HashSet::new();

    for col in b {
        for element in col {
            b_all.insert(element);
        }
    }

    for col in a {
        for element in &col {
            if !b_all.contains(element) {
                return false;
            }
        }
    }

    true
}

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

    let lookup_trace = RawLookupTrace::read_file("/Users/olegfomenko/RustroverProjects/linea-stark-prover/lookup_949.bin");

    let mut cfgs = Vec::new();

    cfgs.push(raw_trace.push_lookup(lookup_trace.clone()));

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
