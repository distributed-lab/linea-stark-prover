mod air;
mod config;

use air::LineaPermutationAIR;
use ark_ff::PrimeField;
use config::*;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_commit::testing::TrivialPcs;
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{prove, verify};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::marker::PhantomData;
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

use corset::cgo::Trace;
use corset::compiler::Constraint;
use corset::{cgo, import};
use corset::column::{Value, ValueBacking};

pub fn generate_permutation_trace<F: Field>(
    a: Vec<Vec<F>>,
    b: Vec<Vec<F>>,
    challenge: F,
    sz: usize,
) -> RowMajorMatrix<F> {
    let mut res: Vec<F> = Vec::new();

    let prev_check = F::ONE;

    for i in 0..sz {
        let mut a_total = F::ONE;
        let mut b_total = F::ONE;

        for j in 0..a.len() {
            res.push(a[j][i].clone());
            a_total = a_total * (a[j][i] + challenge);

            res.push(b[j][i].clone());
            b_total = b_total * (b[j][i] + challenge);
        }

        let b_total_inverse = b_total.inverse();
        let prev_check = prev_check * a_total * b_total_inverse;
        res.push(prev_check);
        res.push(b_total_inverse);
    }

    RowMajorMatrix::new(res, a.len() + b.len() + 2)
}

fn check<F: Field + std::cmp::Ord>(mut a: Vec<Vec<F>>, mut b: Vec<Vec<F>>) {
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

//fn main() -> Result<(), impl Debug> {
fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    let order: Vec<&str> = vec!["mxp.CN", "mxp.CN_perm", "mxp.C_MEM", "mxp.C_MEM_NEW", "mxp.C_MEM_NEW_perm", "mxp.C_MEM_perm", "mxp.STAMP", "mxp.STAMP_perm", "mxp.WORDS", "mxp.WORDS_NEW", "mxp.WORDS_NEW_perm", "mxp.WORDS_perm"];

    let mut corset = cgo::corset_from_file("zkevm.bin").unwrap();
    // import::parse_binary_trace("traces/trace1.lt", &mut corset, false).unwrap();
    import::parse_json_trace("dump.json", &mut corset, true).unwrap();

    let file_content = fs::read_to_string("dump.json").unwrap();
    let v: HashMap<String, Vec<u32>> = serde_json::from_str(&file_content).unwrap();
    let map_field: HashMap<String, Vec<Bls12_377Fr>> = v
        .into_iter()
        .map(|(key, vec_num)| {
            let mut v: Vec<Bls12_377Fr> = vec_num
                .into_iter()
                .filter(|&num| num != 0)
                .map(|num| Bls12_377Fr::from_canonical_u32(num))
                .collect();

            if v.len() != 1160 {
                for _ in 0..(1160 - v.len()) {
                    v.insert(0, Bls12_377Fr::from_canonical_u32(0))
                }
            }

            (
                key,
                v
            )
        })
        .collect();

    let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
    let mut b: Vec<Vec<Bls12_377Fr>> = Vec::new();

     order.iter().enumerate().for_each(|(i, o)| {
        if i % 2 == 0 {
            a.push(map_field[o.clone()].clone());
        } else {
            b.push(map_field[o.clone()].clone());
        }
    });


    // let a = field_columns[0..6].to_vec().clone();
    // let b = field_columns[6..].to_vec().clone();

    assert_eq!(a.len(), b.len(), "trace must have the same sizes");


    check(a, b);


    // -----------------------------------------------------------

    // // TODO: should not be just random
    // let mut rng = thread_rng();
    // let challenge = rng.sample(Standard {});
    // println!("Challenge: {}", challenge);
    //
    // let perm = Perm::new_from_rng(8, 22, &mut rng);
    // let hash = Hash::new(perm.clone());
    //
    // let dft = Dft::default();
    //
    // // TODO: use proper PCS configured with FRI config
    // //let compress = Compress::new(hash.clone());
    // //let val_mmcs = ValMmcs::new(hash.clone(), compress);
    // //let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    // // let fri_config = FriConfig {
    // //     log_blowup: 1,
    // //     log_final_poly_len: 0,
    // //     num_queries: 128,
    // //     proof_of_work_bits: 0,
    // //     mmcs: challenge_mmcs,
    // // };
    // // let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);
    //
    // let pcs = TrivialPcs {
    //     dft,
    //     log_n: trace_len.ilog2() as usize,
    //     _phantom: PhantomData,
    // };
    //
    // let config = Config::new(pcs);
    //
    // let trace = generate_permutation_trace(a, b, challenge, trace_len);
    //
    // let air = LineaPermutationAIR {
    //     width,
    //     check_column_index: width * 2,
    //     inv_column_index: width * 2 + 1,
    // };
    //
    // let mut challenger = Challenger::new(vec![], hash.clone());
    // let proof = prove(&config, &air, &mut challenger, trace, &vec![challenge]);
    //
    // let mut challenger = Challenger::new(vec![], hash.clone());
    // verify(&config, &air, &mut challenger, &proof, &vec![challenge])
}
