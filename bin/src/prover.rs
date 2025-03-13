use crate::config::{ChallengeMmcs, Challenger, Compress, Config, Dft, Hash, Perm, Val, ValMmcs};
use air::{AirConfig, LineaAIR};
use p3_bls12_377_fr::Bls12_377Fr;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_matrix::dense::RowMajorMatrix;
use p3_symmetric::PaddingFreeSponge;
use p3_uni_stark::verify;
use rand::thread_rng;
use trace::global::RawGlobalTrace;
use trace::lookup::RawLookupTrace;
use trace::permutation::RawPermutationTrace;
use trace::range::RawRangeTrace;
use trace::RawProcessedTrace;

pub fn get_air(
    cfgs: &[AirConfig<Bls12_377Fr>],
    t: &RowMajorMatrix<Bls12_377Fr>,
    challenges: Vec<Bls12_377Fr>,
    min_blowup: usize,
) -> (
    LineaAIR<Bls12_377Fr>,
    TwoAdicFriPcs<Val, Dft, ValMmcs, ValMmcs>,
    PaddingFreeSponge<Perm, 3, 2, 1>,
) {
    // TODO: should not be just random
    let mut rng = thread_rng();

    let perm = Perm::new_from_rng(8, 22, &mut rng);
    let hash = Hash::new(perm.clone());

    let dft = Dft::default();

    let compress = Compress::new(hash.clone());

    let val_mmcs = ValMmcs::new(hash.clone(), compress.clone());
    let challenge_mmcs = ChallengeMmcs::new(hash.clone(), compress.clone());

    let fri_config = FriConfig {
        log_blowup: min_blowup,
        log_final_poly_len: 0,
        num_queries: 33,
        proof_of_work_bits: 0, //TODO: 29
        mmcs: challenge_mmcs,
    };

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    println!("Creating LineaAir...");

    let air = LineaAIR::new(cfgs.to_vec(), t.width, challenges);

    (air, pcs, hash)
}

pub fn prove_linea(
    challenges: Vec<Bls12_377Fr>,
    permutation_traces: Vec<RawPermutationTrace>,
    lookup_traces: Vec<RawLookupTrace>,
    range_traces: Vec<RawRangeTrace>,
    global_traces: Vec<RawGlobalTrace>,
    height: usize,
    blowup: usize,
) {
    let mut raw_trace = RawProcessedTrace::new(challenges.clone(), height);
    let cfgs = raw_trace.push_traces(
        permutation_traces,
        lookup_traces,
        range_traces,
        global_traces,
    );

    println!("Generating trace...");

    let t = raw_trace.get_trace();

    let (air, pcs, hash) = get_air(&cfgs, &t, challenges.clone(), blowup);

    let config = Config::new(pcs);

    get_air(&cfgs, &t, challenges, blowup);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Proving...");
    let proof = p3_uni_stark::prove(&config, &air, &mut challenger, t, &vec![]);

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Verification...");

    let ver_result =
        verify(&config, &air, &mut challenger, &proof, &vec![]).map_err(|e| println!("{:?}", e));
    assert!(ver_result.is_ok(), "Verification failed");
}
