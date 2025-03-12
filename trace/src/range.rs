use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;
use p3_air::{Air, BaseAir};
use p3_bls12_377_fr::Bls12_377Fr;
use p3_uni_stark::{SymbolicAirBuilder, SymbolicExpression};
use p3_util::log2_ceil_usize;
use air::LineaAIR;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawRangeTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub a_id: Vec<String>,
    pub b: u64,
    pub name: String,
}

impl RawRangeTrace {
    pub fn read_file(path: &str) -> Result<RawRangeTrace, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawRangeTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        Ok(raw_trace)
    }

    pub fn get_max_height(&self) -> usize {
        let mut height = self.b as usize;
        for i in 0..self.a.len() {
            height = max(height, self.a[i].len())
        }
        height
    }

    pub fn get_min_blowup(&self, air: LineaAIR<Bls12_377Fr>, num_public: usize) -> usize {
        let mut builder = SymbolicAirBuilder::new(0, air.width(), num_public);
        air.eval(&mut builder);
        let symbolic_constraints = builder.constraints();

        let constraint_degree = symbolic_constraints
            .iter()
            .map(SymbolicExpression::degree_multiple)
            .max()
            .unwrap_or(0);

        // Increase log blowup for higher security
        let mut log = log2_ceil_usize(constraint_degree - 1);
        if log == 1 {
            log = 2
        }

        log
    }
}
