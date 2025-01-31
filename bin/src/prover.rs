use crate::config::{ChallengeMmcs, Challenger, Compress, Config, Dft, Hash, Perm, ValMmcs};
use air::LineaAIR;
use p3_bls12_377_fr::Bls12_377Fr;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_uni_stark::verify;
use rand::thread_rng;
use trace::lookup::RawLookupTrace;
use trace::permutation::RawPermutationTrace;
use trace::RawTrace;

pub fn prove_linea(
    challenges: Vec<Bls12_377Fr>,
    permutation_traces: Vec<RawPermutationTrace>,
    lookup_traces: Vec<RawLookupTrace>,
) {
    let mut raw_trace = RawTrace::new(challenges.clone());

    let cfgs = raw_trace.push_traces(permutation_traces, lookup_traces);

    // TODO: should not be just random
    let mut rng = thread_rng();

    let perm = Perm::new_from_rng(8, 22, &mut rng);
    let hash = Hash::new(perm.clone());

    let dft = Dft::default();

    let compress = Compress::new(hash.clone());
    let val_mmcs = ValMmcs::new(hash.clone(), compress.clone());
    let challenge_mmcs = ChallengeMmcs::new(hash.clone(), compress.clone());
    let fri_config = FriConfig {
        log_blowup: 3,
        log_final_poly_len: 0,
        num_queries: 33,
        proof_of_work_bits: 0, //TODO: 29
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
    let proof = p3_uni_stark::prove(&config, &air, &mut challenger, t, &challenges.clone());

    let mut challenger = Challenger::new(vec![], hash.clone());
    println!("Verification...");
    assert!(
        verify(&config, &air, &mut challenger, &proof, &challenges.clone()).is_ok(),
        "Verification failed"
    );
}
