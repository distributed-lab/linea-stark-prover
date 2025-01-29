pub mod lookup;
pub mod permutation;

use crate::lookup::RawLookupTrace;
use crate::permutation::RawPermutationTrace;
use air::air_lookup::AirLookupConfig;
use air::air_permutation::AirPermutationConfig;
use air::AirConfig;
use ark_ff::PrimeField;
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

    pub fn push_lookup(&mut self, lookup: RawLookupTrace) -> AirConfig {
        let mut l = lookup.clone();

        // Determine local maximum height (can not be less than current max height)
        let mut max_height = self.height;

        lookup.a.iter().for_each(|ai| {
            max_height = max(max_height, ai.len());
        });

        lookup.b.iter().for_each(|bi| {
            max_height = max(max_height, bi.len());
        });

        // Resize trace according to the max height
        l.resize(max_height);

        // If max height in our lookup exceeds the current one
        // then we have to resize other constraints
        if max_height > self.height {
            self.resize(max_height);
            self.height = max_height;
        }

        let (mut cfg, mut lookup_columns) = l.get_trace(self.challenges.clone());
        cfg.shift(self.columns.len());
        self.columns.append(&mut lookup_columns);

        AirConfig::Lookup(cfg)
    }

    pub fn push_permutation(&mut self, permutation: RawPermutationTrace) -> AirConfig {
        let mut p = permutation.clone();

        // Determine local maximum height (can not be less than current max height)
        let mut max_height = self.height;

        permutation.a.iter().for_each(|ai| {
            max_height = max(max_height, ai.len());
        });

        permutation.b.iter().for_each(|bi| {
            max_height = max(max_height, bi.len());
        });

        // Resize trace according to the max height
        p.resize(max_height);

        // If max height in our lookup exceeds the current one
        // then we have to resize other constraints
        if max_height > self.height {
            self.resize(max_height);
            self.height = max_height;
        }

        let (mut cfg, mut permutation_columns) = p.get_trace(self.challenges.clone());
        cfg.shift(self.columns.len());
        self.columns.append(&mut permutation_columns);

        AirConfig::Permutation(cfg)
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
