mod air;

use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Add, Neg, Sub};

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, Field};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::extension::BinomialExtensionField;
use p3_fri::FriConfig;
use p3_keccak::Keccak256Hash;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_mersenne_31::Mersenne31;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use p3_uni_stark::{prove, verify, StarkConfig};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use crate::air::LineaAir;



pub fn generate_permutation_trace<F: Field> (init_values: Vec<F>, perm_values: Vec<F>, challenge: F) -> RowMajorMatrix<F> {
    let mut res: Vec<F> = Vec::new();

    init_values.iter().enumerate().for_each(|(i, iv)| {
        let perm = perm_values.get(i).unwrap();

        res.push(iv.clone());
        res.push(perm.clone());

        // t_inv[i] = (u + t[i])^-1
        let inv = challenge.add(*perm).inverse();

        if i != 0 {
            // s[i] = s[i-1] * (u + f[i]) * t_inv[i]
            res.push(res.get(res.len() - 4).unwrap().mul(challenge.add(*iv)).mul(inv));
        } else {
            // s[0] = (u + f[0]) * t_inv[0]
            res.push(challenge.add(*iv).mul(inv));
        }

        res.push(inv);
    });

    RowMajorMatrix::new(res, 4)
}


fn main() -> Result<(), impl Debug> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    type Val = Mersenne31;
    type Challenge = BinomialExtensionField<Val, 3>;

    type ByteHash = Keccak256Hash;
    type FieldHash = SerializingHasher32<ByteHash>;
    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(Keccak256Hash {});

    type MyCompress = CompressionFunctionFromHasher<u8, ByteHash, 2, 32>;
    let compress = MyCompress::new(byte_hash);

    type ValMmcs = FieldMerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
    let val_mmcs = ValMmcs::new(field_hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;

    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 100,
        proof_of_work_bits: 16,
        mmcs: challenge_mmcs,
    };

    type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;
    let pcs = Pcs {
        mmcs: val_mmcs,
        fri_config,
        _phantom: PhantomData,
    };

    type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
    let config = MyConfig::new(pcs);

    let challenge = Mersenne31::from_canonical_u32(10);
    let air = LineaAir { challenge };

    let f0 = Mersenne31::from_canonical_u32(0);
    let f1 = Mersenne31::from_canonical_u32(1);
    let f2 = Mersenne31::from_canonical_u32(2);
    let f3 = Mersenne31::from_canonical_u32(3);

    let trace = generate_permutation_trace(vec![f0, f1, f2, f3], vec![f3, f1, f0, f2], challenge);

    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    let proof = prove(&config, &air, &mut challenger, trace, &vec![]);

    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    verify(&config, &air, &mut challenger, &proof, &vec![])
}
