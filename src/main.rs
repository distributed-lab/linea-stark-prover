mod air;
mod config;
mod trace;

use crate::air::LineaPermutationAIR;
use crate::config::*;
use crate::trace::{generate_permutation_trace, read_trace};
use ark_ff::PrimeField;
use corset::cgo;
use corset::compiler::Constraint;
use p3_bls12_377_fr::Bls12_377Fr;
use p3_commit::testing::TrivialPcs;
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;
use p3_uni_stark::{prove, verify};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::cmp::max;
use std::fmt::Debug;
use std::marker::PhantomData;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

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

    let corset = cgo::corset_from_file("zkevm.bin").unwrap();
    let trace = read_trace("dump.json");

    let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
    let mut b: Vec<Vec<Bls12_377Fr>> = Vec::new();

    let mut trace_len: usize = 0;

    for constraint in corset.constraints {
        match constraint {
            Constraint::Permutation { handle, from, to } => {
                println!("Found a permutation constraint: {:?}", handle);

                for cref in from {
                    let column_name = cref.h.unwrap().to_string();
                    let column = trace.get(&column_name).unwrap().clone();
                    let column_size = column.len();
                    println!(
                        "Found 'from' column: {}, length: {}",
                        column_name, column_size
                    );

                    a.push(column);
                    trace_len = max(column_size, trace_len);
                }

                for cref in to {
                    let column_name = cref.h.unwrap().to_string();
                    let column = trace.get(&column_name).unwrap().clone();
                    let column_size = column.len();
                    println!(
                        "Found 'to' column: {}, length: {}",
                        column_name, column_size
                    );

                    b.push(column);
                    trace_len = max(column_size, trace_len);
                }

                break;
            }
            _ => {}
        }
    }

    assert_eq!(a.len(), b.len(), "trace must have the same sizes");
    let trace_width = a.len();

    assert_ne!(trace_len, 0, "trace length should not be 0");
    trace_len = trace_len.next_power_of_two();

    println!("Appending trace to {}", trace_len);

    for i in 0..trace_width {
        while a[i].len() < trace_len {
            a[i].push(Bls12_377Fr::ZERO);
        }

        while b[i].len() < trace_len {
            b[i].push(Bls12_377Fr::ZERO);
        }
    }

    // dummy_check(a, b);

    // -----------------------------------------------------------

    // Test on small vectors

    // a.push(vec![
    //     Bls12_377Fr::from_canonical_u32(1),
    //     Bls12_377Fr::from_canonical_u32(5),
    //     Bls12_377Fr::from_canonical_u32(4),
    //     Bls12_377Fr::from_canonical_u32(8),
    // ]);
    //
    // b.push(vec![
    //     Bls12_377Fr::from_canonical_u32(1),
    //     Bls12_377Fr::from_canonical_u32(8),
    //     Bls12_377Fr::from_canonical_u32(5),
    //     Bls12_377Fr::from_canonical_u32(4),
    // ]);

    // let trace_len: usize = 4;
    // let trace_width: usize = 1;

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
        log_blowup: 3,
        log_final_poly_len: 0,
        num_queries: 55,
        proof_of_work_bits: 18,
        mmcs: challenge_mmcs,
    };

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    // let pcs = TrivialPcs {
    //     dft,
    //     log_n: trace_len.ilog2() as usize,
    //     _phantom: PhantomData,
    // };

    let config = Config::new(pcs);

    println!("Generating trace...");
    let trace = generate_permutation_trace(a, b, challenge, trace_len);

    let air = LineaPermutationAIR {
        width: trace_width,
        check_column_index: trace_width * 2,
        inv_column_index: trace_width * 2 + 1,
        challenge: challenge.clone(),
    };

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Proving...");
    let proof = prove(&config, &air, &mut challenger, trace, &vec![]);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Verification...");
    verify(&config, &air, &mut challenger, &proof, &vec![])
}
