pub mod lookup;
pub mod lookup_no_filter;
pub mod permutation;

use crate::lookup::RawLookupTrace;
use crate::lookup_no_filter::RawLookupNoFilterTrace;
use crate::permutation::RawPermutationTrace;
use air::air_lookup::AirLookupConfig;
use air::air_permutation::AirPermutationConfig;
use air::AirConfig;
use ark_ff::PrimeField;
use p3_air::Air;
use p3_bls12_377_fr::Bls12_377Fr;
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;

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
    
    pub fn push_lookup(&mut self, lookup: RawLookupTrace) -> AirConfig {
        let mut l = lookup.clone();
        l.resize(self.height);
        
        let cfg = l.update_registry(&mut self.column_registry, &mut self.columns);
        l.set_trace(self.challenges.clone(), &mut self.columns, &cfg);
        AirConfig::Lookup(cfg)
    }

    pub fn push_lookup_no_filter(&mut self, lookup: RawLookupNoFilterTrace) -> AirConfig {
        let mut l = lookup.clone();
        l.resize(self.height);
        
        let cfg = l.update_registry(&mut self.column_registry, &mut self.columns);
        l.set_trace(self.challenges.clone(), &mut self.columns, &cfg);
        AirConfig::LookupNoFilters(cfg)
    }

    pub fn push_permutation(&mut self, permutation: RawPermutationTrace) -> AirConfig {
        let mut p = permutation.clone();
        p.resize(self.height);
        
        let cfg = p.update_registry(&mut self.column_registry, &mut self.columns);
        p.set_trace(self.challenges.clone(), &mut self.columns, &cfg);
        AirConfig::Permutation(cfg)
    }

    pub fn push_traces(
        &mut self,
        permutation_traces: Vec<RawPermutationTrace>,
        lookup_traces: Vec<RawLookupTrace>,
        lookup_no_filter_traces: Vec<RawLookupNoFilterTrace>,
    ) -> Vec<AirConfig> {
        let mut cfgs = Vec::new();

        lookup_traces.iter().for_each(|lt| {
            cfgs.push(self.push_lookup(lt.clone()));
        });

        permutation_traces.iter().for_each(|pt| {
            cfgs.push(self.push_permutation(pt.clone()));
        });

        lookup_no_filter_traces.iter().for_each(|lt| {
            cfgs.push(self.push_lookup_no_filter(lt.clone()));
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
