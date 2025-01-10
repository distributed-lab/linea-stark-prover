mod air;
mod config;

use air::{LineaPermutationAIR, PERMUTATION_WIDTH};
use config::*;

use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::fmt::Debug;
use std::marker::PhantomData;

use p3_bls12_377_fr::Bls12_377Fr;
use p3_commit::testing::TrivialPcs;
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{prove, verify};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn generate_permutation_trace<F: Field>(
    column_a: Vec<F>,
    column_b: Vec<F>,
    challenge: F,
) -> RowMajorMatrix<F> {
    let mut res: Vec<F> = Vec::new();

    res.push(column_a[0].clone());
    res.push(column_b[0].clone());

    let mut inverse = (column_b[0] + challenge).inverse();
    let mut previous = (column_a[0] + challenge) * inverse;
    res.push(previous);
    res.push(inverse);

    column_a[1..]
        .iter()
        .zip(column_b[1..].iter())
        .for_each(|(a, b)| {
            res.push(a.clone());
            res.push(b.clone());
            inverse = (*b + challenge).inverse();
            previous = (*a + challenge) * inverse * previous;
            res.push(previous);
            res.push(inverse);
        });

    RowMajorMatrix::new(res, PERMUTATION_WIDTH)
}

fn main() -> Result<(), impl Debug> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // TODO: should not be just random
    let mut rng = thread_rng();
    let challenge = rng.sample(Standard {});
    println!("Challenge: {}", challenge);

    let log_n: usize = 2;
    let a = vec![
        Bls12_377Fr::from_canonical_u32(1),
        Bls12_377Fr::from_canonical_u32(2),
        Bls12_377Fr::from_canonical_u32(3),
        Bls12_377Fr::from_canonical_u32(4),
    ];

    let b = vec![
        Bls12_377Fr::from_canonical_u32(3),
        Bls12_377Fr::from_canonical_u32(1),
        Bls12_377Fr::from_canonical_u32(4),
        Bls12_377Fr::from_canonical_u32(2),
    ];

    let perm = Perm::new_from_rng(8, 22, &mut rng);
    let hash = Hash::new(perm.clone());

    let dft = Dft::default();

    // TODO: use proper PCS configured with FRI config
    //let compress = Compress::new(hash.clone());
    //let val_mmcs = ValMmcs::new(hash.clone(), compress);
    //let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    // let fri_config = FriConfig {
    //     log_blowup: 1,
    //     log_final_poly_len: 0,
    //     num_queries: 128,
    //     proof_of_work_bits: 0,
    //     mmcs: challenge_mmcs,
    // };
    // let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    let pcs = TrivialPcs {
        dft,
        log_n,
        _phantom: PhantomData,
    };

    let config = Config::new(pcs);

    let trace = generate_permutation_trace(a, b, challenge);

    let mut challenger = Challenger::new(vec![], hash.clone());
    let proof = prove(
        &config,
        &LineaPermutationAIR {},
        &mut challenger,
        trace,
        &vec![challenge],
    );

    let mut challenger = Challenger::new(vec![], hash.clone());
    verify(
        &config,
        &LineaPermutationAIR {},
        &mut challenger,
        &proof,
        &vec![challenge],
    )
}
