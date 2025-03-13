use crate::lookup::RawLookupTrace;
use crate::{RawProcessedTrace, RawTrace, MIN_LOG_BLOWUP};
use air::{AirConfig, LineaAIR};
use p3_air::{Air, BaseAir};
use p3_bls12_377_fr::Bls12_377Fr;
use p3_uni_stark::{SymbolicAirBuilder, SymbolicExpression};
use p3_util::log2_ceil_usize;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawRangeTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub a_id: Vec<String>,
    pub b: u64,
    pub name: String,
}

impl RawTrace for RawRangeTrace {
    fn read_file(path: &str) -> Result<Self, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawRangeTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        Ok(raw_trace)
    }

    fn get_max_height(&self) -> usize {
        let mut height = self.a[0].len();
        for i in 0..self.a.len() {
            height = max(height, self.a[i].len())
        }
        height
    }

    fn get_min_blowup(&self, air: LineaAIR<Bls12_377Fr>, num_public: usize) -> usize {
        let mut builder = SymbolicAirBuilder::new(0, air.width(), num_public);
        air.eval(&mut builder);
        let symbolic_constraints = builder.constraints();

        let constraint_degree = symbolic_constraints
            .iter()
            .map(SymbolicExpression::degree_multiple)
            .max()
            .unwrap_or(0);

        // Increase log blowup for higher security
        max(log2_ceil_usize(constraint_degree - 1), MIN_LOG_BLOWUP)
    }

    fn update_processed_trace(
        &mut self,
        processed: &mut RawProcessedTrace,
    ) -> AirConfig<Bls12_377Fr> {
        let mut l = RawLookupTrace::from(self.clone());
        l.resize(processed.height);

        l.update_processed_trace(processed)
    }
}
