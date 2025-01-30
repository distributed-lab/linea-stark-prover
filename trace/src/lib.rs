pub mod lookup;
pub mod permutation;
pub mod range;

use crate::lookup::RawLookupTrace;
use crate::permutation::RawPermutationTrace;
use crate::range::RawRangeTrace;
use air::AirConfig;
use ark_ff::PrimeField;
use p3_air::Air;
use p3_bls12_377_fr::Bls12_377Fr;
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use serde::{Deserialize, Serialize};
use std::cmp::max;

pub struct RawTrace {
    pub columns: Vec<Vec<Bls12_377Fr>>,
    pub height: usize,
    pub challenges: Vec<Bls12_377Fr>,
}

impl RawTrace {
    pub fn new(challenges: Vec<Bls12_377Fr>) -> Self {
        RawTrace {
            columns: vec![],
            height: 0,
            challenges,
        }
    }
    pub fn resize(&mut self, new_size: usize) {
        for e in &mut self.columns {
            e.resize(new_size, Bls12_377Fr::ZERO);
        }
    }

    pub fn push_range(&mut self, range: RawRangeTrace) -> AirConfig {
        let mut r = range.clone();

        // Resize trace according to the max height
        r.resize(self.height);

        let (mut cfg, mut lookup_columns) = r.get_trace(self.challenges.clone());
        cfg.shift(self.columns.len());
        self.columns.append(&mut lookup_columns);

        AirConfig::Lookup(cfg)
    }

    pub fn push_lookup(&mut self, lookup: RawLookupTrace) -> AirConfig {
        let mut l = lookup.clone();

        // Resize trace according to the max height
        l.resize(self.height);

        let (mut cfg, mut lookup_columns) = l.get_trace(self.challenges.clone());
        cfg.shift(self.columns.len());
        self.columns.append(&mut lookup_columns);

        AirConfig::Lookup(cfg)
    }

    pub fn push_permutation(&mut self, permutation: RawPermutationTrace) -> AirConfig {
        let mut p = permutation.clone();
        // Resize trace according to the max height
        p.resize(self.height);

        let (mut cfg, mut permutation_columns) = p.get_trace(self.challenges.clone());
        cfg.shift(self.columns.len());
        self.columns.append(&mut permutation_columns);

        AirConfig::Permutation(cfg)
    }

    pub fn push_traces(
        &mut self,
        permutation_traces: Vec<RawPermutationTrace>,
        lookup_traces: Vec<RawLookupTrace>,
        range_traces: Vec<RawRangeTrace>,
    ) -> Vec<AirConfig> {
        // Get max height of all range traces.
        let mut range_max_height = 0;
        range_traces.iter().for_each(|rt| {
            range_max_height = max(range_max_height, rt.get_max_height());
        });

        // Get max height of all lookup traces.
        let mut lookup_max_height = 0;
        lookup_traces.iter().for_each(|lt| {
            lookup_max_height = max(lookup_max_height, lt.get_max_height());
        });

        // Get max height of all permutation traces.
        let mut permutation_max_height = 0;
        permutation_traces.iter().for_each(|pt| {
            permutation_max_height = max(permutation_max_height, pt.get_max_height());
        });

        // Get trace max height.
        self.height = max(
            max(permutation_max_height, lookup_max_height),
            range_max_height,
        );

        let mut cfgs = Vec::new();
        range_traces.iter().for_each(|rt| {
            cfgs.push(self.push_range(rt.clone()));
        });

        lookup_traces.iter().for_each(|lt| {
            cfgs.push(self.push_lookup(lt.clone()));
        });

        permutation_traces.iter().for_each(|pt| {
            cfgs.push(self.push_permutation(pt.clone()));
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
