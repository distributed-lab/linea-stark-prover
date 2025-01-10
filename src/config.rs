use p3_bls12_377_fr::{Bls12_377Fr, Poseidon2Bls12337};
use p3_challenger::HashChallenger;
use p3_commit::testing::TrivialPcs;
use p3_dft::Radix2DitParallel;
use p3_symmetric::PaddingFreeSponge;
use p3_uni_stark::StarkConfig;

pub type Val = Bls12_377Fr;
pub type Challenge = Bls12_377Fr;
pub type Perm = Poseidon2Bls12337<3>;
pub type Hash = PaddingFreeSponge<Perm, 3, 2, 1>;

// Use with proper PCS
/// Defines a compression function type using ByteHash, with 2 input blocks and 32-byte output.
///
// pub type Compress = CompressionFunctionFromHasher<Hash, 2, 1>;
//pub type Compress = TruncatedPermutation<Perm, 2, 1, 3>;
// pub type ValMmcs =
//     MerkleTreeMmcs<<Val as Field>::Packing, <Val as Field>::Packing, Hash, Compress, 1>;
// pub type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;

pub type Dft = Radix2DitParallel<Val>;
pub type Challenger = HashChallenger<Val, Hash, 1>;
pub type Config = StarkConfig<TrivialPcs<Val, Dft>, Challenge, Challenger>;
