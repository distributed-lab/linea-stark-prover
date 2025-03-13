pub mod global;
pub mod lookup;
pub mod permutation;
pub mod range;

use crate::global::RawGlobalTrace;
use crate::lookup::RawLookupTrace;
use crate::permutation::RawPermutationTrace;
use crate::range::RawRangeTrace;
use air::{AirConfig, LineaAIR};
use p3_bls12_377_fr::Bls12_377Fr;
use p3_field::FieldAlgebra;
use p3_matrix::dense::RowMajorMatrix;
use std::collections::HashMap;

pub trait RawTrace: Sized {
    fn read_file(path: &str) -> Result<Self, std::io::Error>;
    fn get_max_height(&self) -> usize;
    fn get_min_blowup(&self, air: LineaAIR<Bls12_377Fr>, num_public: usize) -> usize;
    fn update_processed_trace(
        &mut self,
        processed: &mut RawProcessedTrace,
    ) -> AirConfig<Bls12_377Fr>;
}

pub struct RawProcessedTrace {
    pub columns: Vec<Vec<Bls12_377Fr>>,
    pub height: usize,
    pub challenges: Vec<Bls12_377Fr>,
    pub column_registry: HashMap<String, usize>,
}

impl RawProcessedTrace {
    pub fn new(challenges: Vec<Bls12_377Fr>, height: usize) -> Self {
        RawProcessedTrace {
            columns: vec![],
            height,
            challenges,
            column_registry: HashMap::new(),
        }
    }

    pub fn resize(&mut self, new_size: usize) {
        for e in &mut self.columns {
            e.resize(new_size, Bls12_377Fr::ZERO);
        }
    }

    pub fn push_traces(
        &mut self,
        permutation_traces: Vec<RawPermutationTrace>,
        lookup_traces: Vec<RawLookupTrace>,
        range_traces: Vec<RawRangeTrace>,
        global_traces: Vec<RawGlobalTrace>,
    ) -> Vec<AirConfig<Bls12_377Fr>> {
        let mut cfgs = Vec::new();

        lookup_traces.into_iter().for_each(|mut lt| {
            cfgs.push(lt.update_processed_trace(self));
        });

        range_traces.into_iter().for_each(|mut rt| {
            cfgs.push(rt.update_processed_trace(self));
        });

        permutation_traces.into_iter().for_each(|mut pt| {
            cfgs.push(pt.update_processed_trace(self));
        });

        global_traces.into_iter().for_each(|mut gb| {
            cfgs.push(gb.update_processed_trace(self));
        });

        cfgs
    }

    pub fn get_trace(&mut self) -> RowMajorMatrix<Bls12_377Fr> {
        let width = self.columns.len();
        // The final trace
        let mut values = vec![];

        for row in 0..self.height {
            for col in 0..width {
                values.push(std::mem::take(&mut self.columns[col][row]));
            }
        }

        RowMajorMatrix::new(values, width)
    }
}
