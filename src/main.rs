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

    let mut corset = cgo::corset_from_file("zkevm.bin").unwrap();
    // import::parse_binary_trace("traces/trace1.lt", &mut corset, true).unwrap();
    import::parse_json_trace("dump.json", &mut corset, true).unwrap();

    // let file_content = fs::read_to_string("dump.json").unwrap();
    // let v: HashMap<String, Vec<u32>> = serde_json::from_str(&file_content).unwrap();
    // let map_field: HashMap<String, Vec<Bls12_377Fr>> = v
    //     .into_iter()
    //     .map(|(key, vec_u32)| (key, vec_u32.into_iter().map(|num| Bls12_377Fr::from_canonical_u32(num)).collect()))
    //

    let file_content = fs::read_to_string("dump.json").unwrap();
    let v: HashMap<String, Vec<u32>> = serde_json::from_str(&file_content).unwrap();
    let map_field: HashMap<String, Vec<Bls12_377Fr>> = v
        .into_iter()
        .map(|(key, vec_num)| {
            (
                key,
                vec_num
                    .into_iter()
                    .filter(|&num| num != 0)
                    .map(|num| Bls12_377Fr::from_canonical_u32(num))
                    .collect(),
            )
        })
        .collect();

    map_field.iter().for_each(|(_, a)| println!("{}", a.len()));


    // let trace = Trace::from_constraints(&corset);
    // let mut indexes: HashMap<String, usize> = HashMap::new();
    // for i in 0..trace.columns.len() {
    //     indexes.insert(trace.ids[i].clone(), i);
    // }
    //
    // //println!("{:?}", corset.columns.cols);
    //
    // let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
    // let mut b: Vec<Vec<Bls12_377Fr>> = Vec::new();
    //
    // let mut trace_len = 0;
    //
    // for constraint in corset.constraints {
    //     match constraint {
    //         Constraint::Permutation { handle, from, to } => {
    //             println!("found permutation constraint {:?}", handle);
    //
    //             for cref in from {
    //                 let mut column: Vec<Bls12_377Fr> = Vec::new();
    //                 //println!("ref A {:?}", cref);
    //                 let column_name = cref.h.unwrap().to_string();
    //                 println!("Column A name: {}", column_name);
    //                 let id = *indexes.get(&column_name).unwrap();
    //
    //                 let trace_column = &trace.columns[id];
    //
    //                 // TODO: should we use it
    //                 let padding_value = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
    //                     &trace_column.padding_value,
    //                 ));
    //
    //                 // let actual_size = trace_column.values.len();
    //                 // let target_size = actual_size.next_power_of_two();
    //                 // for _ in 0..(target_size - actual_size) {
    //                 //     column.push(padding_value.clone());
    //                 // }
    //
    //
    //                 //println!("trace column name {:?}", trace.ids[cref.id.unwrap()]);
    //                 println!("padding value in A {:?}", padding_value);
    //                 for value in &trace_column.values {
    //                     column.push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
    //                         value.as_slice(),
    //                     )));
    //                 }
    //
    //                 a.push(column.clone());
    //                 trace_len = max(trace_column.values.len(), trace_len);
    //                 println!("{}", column.len());
    //             }
    //
    //             // for cref in to {
    //             //     let column = corset.columns._cols.get(cref.as_id()).unwrap();
    //             //     let reg_id = column.register.unwrap();
    //             //     let reg = corset.columns.registers.get(reg_id).unwrap();
    //             //     println!("{:?}", reg);
    //             //     // let mut column: Vec<Bls12_377Fr> = Vec::new();
    //             //     // let column_name = cref.h.unwrap().to_string();
    //             //     // println!("Column B name: {}", column_name);
    //             //     // let id = *indexes.get(&column_name).unwrap();
    //             //     //
    //             //     // let trace_column = &trace.columns[id];
    //             //     // println!("Column B: {:?}", trace_column.values);
    //             //     //
    //             //     // // TODO: should we use it
    //             //     // let padding_value = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
    //             //     //     &trace_column.padding_value,
    //             //     // ));
    //             //     // // let actual_size = trace_column.values.len();
    //             //     // // let target_size = actual_size.next_power_of_two();
    //             //     // // for _ in 0..(target_size - actual_size) {
    //             //     // //     column.push(padding_value.clone());
    //             //     // // }
    //             //     //
    //             //     // // println!("ref B {:?}", cref);
    //             //     // // println!("trace column name {:?}", trace.ids[cref.id.unwrap()]);
    //             //     // println!("padding value in B {:?}", padding_value);
    //             //     //
    //             //     // for value in &trace_column.values {
    //             //     //     column.push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
    //             //     //         value.as_slice(),
    //             //     //     )));
    //             //     // }
    //             //     //
    //             //     // b.push(column);
    //             //     // trace_len = max(trace_column.values.len(), trace_len);
    //             // }
    //
    //             break;
    //         }
    //         _ => {}
    //     }
    // }

    // assert_ne!(trace_len, 0, "trace is empty");

    // assert_eq!(a.len(), b.len(), "trace must have the same sizes");
    // let width = a.len();

    // trace_len = trace_len.next_power_of_two();

    // a.reverse().iter().for_each(|v| println!("{}", v.len()));
    // println!("{:?}", b.len());
    //
    // // for i in 0..a.len() {
    // //     while a[i].len() < trace_len {
    // //         a[i].push(Bls12_377Fr::ZERO);
    // //     }
    // // }
    // //
    // // for i in 0..b.len() {
    // //     while b[i].len() < trace_len {
    // //         b[i].push(Bls12_377Fr::ZERO);
    // //     }
    // // }
    //
    // for i in 0..width {
    //     while a[i].len() < trace_len {
    //         a[i].push(Bls12_377Fr::ZERO);
    //     }
    //
    //     while b[i].len() < trace_len {
    //         b[i].push(Bls12_377Fr::ZERO);
    //     }
    // }
    //
    // check(a, b);

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
