// use air::{AirConfig, AirLookupConfig};
use air::air_lookup::AirLookupConfig;
use air::air_permutation::AirPermutationConfig;
use air::AirConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawPermutationTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub b: Vec<Vec<[u8; 32]>>,
    pub name: String,
}

impl RawPermutationTrace {
    pub fn a_width(&self) -> usize {
        self.a.len()
    }

    pub fn read_file(path: &str) -> Self {
        let file_content = fs::read(path).unwrap();
        let raw_trace: RawPermutationTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();
        raw_trace
    }

    pub fn get_trace(&self, challenge: Bls12_377Fr) -> RowMajorMatrix<Bls12_377Fr> {
        let sz = self.a[0].len();
        let width = self.a.len();
        let mut res: Vec<Bls12_377Fr> = Vec::new();
        let mut prev_check = Bls12_377Fr::ONE;

        for i in 0..sz {
            let mut a_total = Bls12_377Fr::ONE;
            let mut b_total = Bls12_377Fr::ONE;

            for j in 0..self.a.len() {
                let aji = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.a[j][i].as_slice(),
                ));
                a_total *= aji + challenge;
                res.push(aji);
            }

            for j in 0..self.b.len() {
                let bji = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.b[j][i].as_slice(),
                ));
                b_total *= bji + challenge;
                res.push(bji);
            }

            let b_total_inverse = b_total.inverse();
            prev_check = prev_check * a_total * b_total_inverse;
            res.push(prev_check);
            res.push(b_total_inverse);
        }

        assert!(
            res.get(res.len() - 2).unwrap().is_one(),
            "failed to check constrain: check column should be 1 on the last row"
        );

        RowMajorMatrix::new(res, width * 2 + 2)
    }

    pub fn resize(&mut self, size: usize) {
        for e in &mut self.a {
            e.resize(size, [0u8; 32]);
        }

        for e in &mut self.b {
            e.resize(size, [0u8; 32]);
        }

        // TODO: resize filters
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawLookupTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub b: Vec<Vec<[u8; 32]>>,
    pub name: String,
    pub a_filter: Vec<[u8; 32]>,
    pub b_filter: Vec<[u8; 32]>,
}

impl RawLookupTrace {
    pub fn read_file(path: &str) -> Self {
        let file_content = fs::read(path).unwrap();
        let raw_trace: RawLookupTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        raw_trace
    }

    pub fn a_width(&self) -> usize {
        self.a.len()
    }

    pub fn b_width(&self) -> usize {
        self.b.len()
    }

    pub(crate) fn get_trace(&self, challenge: Bls12_377Fr) -> RowMajorMatrix<Bls12_377Fr> {
        let (a, b, mut a_occurrences) = self.get_columns();

        let mut res: Vec<Bls12_377Fr> = Vec::new();
        let sz = a[0].len();
        let mut sum = Bls12_377Fr::ZERO;

        for i in 0..sz {
            let mut a_inverses = Vec::new();

            for col in &a {
                let aji_inv = (col[i] + challenge).inverse();
                sum += aji_inv;
                a_inverses.push(aji_inv);
                res.push(col[i]);
            }

            res.append(&mut a_inverses);

            let mut b_inverses = Vec::new();
            let mut b_occurrences: Vec<Bls12_377Fr> = Vec::new();

            for col in &b {
                let bji_inv = (col[i] + challenge).inverse();

                let mut occurrence = Bls12_377Fr::from_canonical_usize(0);

                if let Some(cnt) = a_occurrences.get(&col[i]) {
                    occurrence = Bls12_377Fr::from_canonical_usize(*cnt);
                    sum -= bji_inv * occurrence;
                    a_occurrences.insert(col[i], 0);
                }

                b_occurrences.push(occurrence);
                b_inverses.push(bji_inv);
                res.push(col[i]);
            }

            res.append(&mut b_inverses);
            res.append(&mut b_occurrences);
            res.push(sum);
        }

        assert!(
            res.last().unwrap().is_zero(),
            "failed to check constrain: check column should be 0 on the last row"
        );
        RowMajorMatrix::new(res, 2 * a.len() + 3 * b.len() + 1)
    }

    pub fn resize(&mut self, size: usize) {
        for e in &mut self.a {
            e.resize(size, [0u8; 32]);
        }

        for e in &mut self.b {
            e.resize(size, [0u8; 32]);
        }

        // TODO: resize filters
    }

    pub fn get_columns(
        &self,
    ) -> (
        Vec<Vec<Bls12_377Fr>>,
        Vec<Vec<Bls12_377Fr>>,
        HashMap<Bls12_377Fr, usize>,
    ) {
        let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
        let mut b: Vec<Vec<Bls12_377Fr>> = Vec::new();

        let mut a_occurrences: HashMap<Bls12_377Fr, usize> = HashMap::new();

        for i in 0..self.a.len() {
            a.push(Vec::new());
            for j in 0..self.a[i].len() {
                let aij = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.a[i][j].as_slice(),
                ));

                a[i].push(aij);

                if let Some(cnt) = a_occurrences.get(&aij) {
                    a_occurrences.insert(aij, cnt + 1);
                } else {
                    a_occurrences.insert(aij, 1);
                }
            }
        }

        for i in 0..self.b.len() {
            b.push(Vec::new());

            for j in 0..self.b[i].len() {
                let bij = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.b[i][j].as_slice(),
                ));

                b[i].push(bij);
            }
        }

        (a, b, a_occurrences)
    }
}

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

    // TODO: rewrite it
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
