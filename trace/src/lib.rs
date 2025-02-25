pub mod global;
pub mod lookup;
pub mod permutation;
pub mod range;

use crate::global::RawGlobalTrace;
use crate::lookup::RawLookupTrace;
use crate::permutation::RawPermutationTrace;
use crate::range::RawRangeTrace;
use air::AirConfig;
use p3_bls12_377_fr::Bls12_377Fr;
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use std::collections::HashMap;
use p3_air::AirBuilder;

pub struct RawTrace {
    pub columns: Vec<Vec<Bls12_377Fr>>,
    pub height: usize,
    pub challenges: Vec<Bls12_377Fr>,
    pub column_registry: HashMap<String, usize>,
}

impl RawTrace {
    pub fn new(challenges: Vec<Bls12_377Fr>, height: usize) -> Self {
        RawTrace {
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

    pub fn push_lookup(&mut self, lookup: &mut RawLookupTrace) -> AirConfig<Bls12_377Fr> {
        lookup.resize(self.height);

        let cfg = lookup.update_registry(&mut self.column_registry, &mut self.columns);

        lookup.set_trace(self.challenges.clone(), &mut self.columns, &cfg);
        AirConfig::Lookup(cfg)
    }

    pub fn push_range(&mut self, range: RawRangeTrace) -> AirConfig<Bls12_377Fr> {
        let mut l = RawLookupTrace::from(range);
        l.resize(self.height);

        self.push_lookup(&mut l)
    }

    pub fn push_permutation(&mut self, permutation: &mut RawPermutationTrace) -> AirConfig<Bls12_377Fr> {
        permutation.resize(self.height);

        let cfg = permutation.update_registry(&mut self.column_registry, &mut self.columns);
        permutation.set_trace(self.challenges.clone(), &mut self.columns, &cfg);
        AirConfig::Permutation(cfg)
    }

    pub fn push_global(&mut self, global: &mut RawGlobalTrace) -> AirConfig<Bls12_377Fr> {
        global.resize(self.height);

        let cfg = global.update_registry(&mut self.column_registry, &mut self.columns);
        global.set_trace(self.challenges.clone(), &mut self.columns, &cfg);
        AirConfig::Global(cfg)
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
            cfgs.push(self.push_lookup(&mut lt));
        });

        range_traces.into_iter().for_each(|rt| {
            cfgs.push(self.push_range(rt));
        });

        permutation_traces.into_iter().for_each(|mut pt| {
            cfgs.push(self.push_permutation(&mut pt));
        });

        global_traces.into_iter().for_each(|mut gb| {
            cfgs.push(self.push_global(&mut gb));
        });

        cfgs
    }

    pub fn get_trace(&self) -> RowMajorMatrix<Bls12_377Fr> {
        let width = self.columns.len();
        // The final trace
        let mut values = vec![];

        for row in 0..self.height {
            for col in 0..width {
                values.push(self.columns[col][row]);
            }
        }

        RowMajorMatrix::new(values, width)
    }
}
