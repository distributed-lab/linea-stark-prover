pub mod lookup;
pub mod permutation;

use air::air_lookup::AirLookupConfig;
use air::air_permutation::AirPermutationConfig;
use air::AirConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use serde::{Deserialize, Serialize};
use crate::lookup::RawLookupTrace;
use crate::permutation::RawPermutationTrace;

pub enum Constraint {
    Permutation(RawPermutationTrace),
    Lookup(RawLookupTrace),
}

impl Constraint {
    pub fn resize(&mut self, new_size: usize) {
        match self {
            Constraint::Permutation(p) => p.resize(new_size),
            Constraint::Lookup(l) => l.resize(new_size),
        };
    }
}

#[derive(Default)]
pub struct RawTrace {
    pub constraints: Vec<Constraint>,
    pub max_height: usize,
}

impl RawTrace {
    pub fn push_lookup(&mut self, lookup: RawLookupTrace) {
        let mut l = lookup.clone();

        match lookup.a[0].len().cmp(&lookup.b[0].len()) {
            std::cmp::Ordering::Greater => {
                let new_size = lookup.a[0].len();
                l.b.iter_mut().for_each(|e| {
                    e.resize(new_size, [0_u8; 32]);
                });
            }
            std::cmp::Ordering::Less => {
                let new_size = lookup.b[0].len();
                l.a.iter_mut().for_each(|e| {
                    e.resize(new_size, [0_u8; 32]);
                });
            }
            std::cmp::Ordering::Equal => {}
        }

        match l.a[0].len().cmp(&self.max_height) {
            std::cmp::Ordering::Greater => {
                let new_size = l.a[0].len();
                self.constraints.iter_mut().for_each(|c| c.resize(new_size));
                self.max_height = new_size;
                self.constraints.push(Constraint::Lookup(l));
            }
            std::cmp::Ordering::Less => {
                let new_size = self.max_height;
                l.resize(new_size);
                self.constraints.push(Constraint::Lookup(l));
            }
            std::cmp::Ordering::Equal => {
                self.constraints.push(Constraint::Lookup(l));
            }
        }
    }

    // TODO: refactor
    pub fn push_permutation(&mut self, permutation: RawPermutationTrace) {
        let mut p = permutation.clone();

        match permutation.a[0].len().cmp(&permutation.b[0].len()) {
            std::cmp::Ordering::Greater => {
                let new_size = permutation.a[0].len();
                p.b.iter_mut().for_each(|e| {
                    e.resize(new_size, [0_u8; 32]);
                });
            }
            std::cmp::Ordering::Less => {
                let new_size = permutation.b[0].len();
                p.a.iter_mut().for_each(|e| {
                    e.resize(new_size, [0_u8; 32]);
                });
            }
            std::cmp::Ordering::Equal => {}
        }

        match p.a[0].len().cmp(&self.max_height) {
            std::cmp::Ordering::Greater => {
                let new_size = p.a[0].len();
                self.constraints.iter_mut().for_each(|c| c.resize(new_size));
                self.max_height = new_size;
                self.constraints.push(Constraint::Permutation(p));
            }
            std::cmp::Ordering::Less => {
                let new_size = self.max_height;
                p.resize(new_size);
                self.constraints.push(Constraint::Permutation(p));
            }
            std::cmp::Ordering::Equal => {
                self.constraints.push(Constraint::Permutation(p));
            }
        }
    }

    pub fn get_trace(&self, challange: Bls12_377Fr) -> RowMajorMatrix<Bls12_377Fr> {
        let mut matrixes = vec![];
        let mut final_width = 0;

        for c in &self.constraints {
            let matrix = match c {
                Constraint::Lookup(l) => l.get_trace(challange),
                Constraint::Permutation(p) => p.get_trace(challange),
            };

            final_width += matrix.width;
            matrixes.push(matrix);
        }

        let max_height = self.max_height;

        let mut values = vec![];

        for row in 0..max_height {
            for matrix in &matrixes {
                let rows = matrix.values.len() / matrix.width;
                if row < rows {
                    // Append the current row's data
                    let start = row * matrix.width;
                    let end = usize::min(start + matrix.width, matrix.values.len());
                    values.append(&mut matrix.values[start..end].to_vec());
                }
            }
        }

        RowMajorMatrix::new(values, final_width)
    }

    pub fn get_air_configs(&self) -> Vec<AirConfig> {
        self.constraints
            .iter()
            .map(|c| match c {
                Constraint::Permutation(p) => {
                    AirConfig::Permutation(AirPermutationConfig::new(p.a_width()))
                }
                Constraint::Lookup(l) => {
                    AirConfig::Lookup(AirLookupConfig::new(l.a_width(), l.b_width()))
                }
            })
            .collect()
    }
}
