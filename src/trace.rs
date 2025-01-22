use crate::config::Challenge;
use ark_ff::PrimeField;
use num_bigint::BigUint;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use p3_matrix::Matrix;
use crate::air::{AirConfig, AirLookupConfig};

#[derive(Serialize, Deserialize, Debug)]
pub struct RawPermutationTrace {
    pub a: Vec<Vec<Vec<u8>>>,
    pub b: Vec<Vec<Vec<u8>>>,
    pub name: String,
}

enum Constraint {
    Permutation(RawPermutationTrace),
    Lookup(RawLookupTrace),
}

impl RawPermutationTrace {
    pub fn column_width(&self) -> usize {
        self.a.len()
    }

    fn get_trace(&self, challenge: Bls12_377Fr) -> RowMajorMatrix<Bls12_377Fr> {
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
                a_total = a_total * (aji + challenge);
                res.push(aji);
            }

            for j in 0..self.b.len() {
                let bji = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    &self.b[j][i].as_slice(),
                ));
                b_total = b_total * (bji + challenge);
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
}

impl RawLookupTrace {
    pub fn a_width(&self) -> usize {
        self.a.len()
    }

    pub fn b_width(&self) -> usize {
        self.b.len()
    }

    pub(crate) fn get_trace(&self, challenge: Bls12_377Fr) -> RowMajorMatrix<Bls12_377Fr> {
        let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
        let mut b: Vec<Vec<Bls12_377Fr>> = Vec::new();

        let mut a_occurrences: HashMap<Bls12_377Fr, usize> = HashMap::new();

        for i in 0..self.a.len() {
            a.push(Vec::new());
            for j in 0..self.a[i].len() {
                let aij = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.a[i][j].as_slice(),
                ));

                a[i].push(aij.clone());

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

                b[i].push(bij.clone());
            }
        }

        let mut res: Vec<Bls12_377Fr> = Vec::new();
        let sz = a[0].len();
        let mut sum = Bls12_377Fr::ZERO;

        for i in 0..sz {
            let mut a_inverses = Vec::new();

            for j in 0..a.len() {
                let aji_inv = (a[j][i] + challenge).inverse();
                sum = sum + aji_inv;
                a_inverses.push(aji_inv);
                res.push(a[j][i]);
            }

            res.append(&mut a_inverses);

            let mut b_inverses = Vec::new();
            let mut b_occurrences: Vec<Bls12_377Fr> = Vec::new();

            for j in 0..b.len() {
                let bji_inv = (b[j][i] + challenge).inverse();

                let mut occurrence = Bls12_377Fr::from_canonical_usize(0);

                if let Some(cnt) = a_occurrences.get(&b[j][i]) {
                    occurrence = Bls12_377Fr::from_canonical_usize(*cnt);
                    sum = sum - bji_inv * occurrence;
                    a_occurrences.insert(b[j][i], 0);
                }

                b_occurrences.push(occurrence);
                b_inverses.push(bji_inv);
                res.push(b[j][i]);
            }

            res.append(&mut b_inverses);
            res.append(&mut b_occurrences);
            res.push(sum.clone());
        }

        assert!(
            res.get(res.len() - 1).unwrap().is_zero(),
            "failed to check constrain: check column should be 0 on the last row"
        );
        RowMajorMatrix::new(res, 2 * a.len() + 3 * b.len() + 1)
    }
}

#[derive(Default)]
pub struct RawTrace {
    pub constraints: Vec<Constraint>
}

impl RawTrace {
    pub fn new(constraints: Vec<Constraint>) -> Self {
        Self{
            constraints
        }
    }

    pub fn push_lookup(&mut self, lookup: RawLookupTrace) {
        self.constraints.push(Constraint::Lookup(lookup))
    }

    pub fn get_trace(&self, challange: Bls12_377Fr) -> RowMajorMatrix<Bls12_377Fr> {
        // let mut values = vec![];
        // let mut final_width = 0;

        // for c in &self.constraints {
        //
        //     let mut matrix = match c {
        //         // Constraint::Permutation(p) => p.get_trace(challange),
        //         Constraint::Lookup(l) => { l.get_trace(challange) },
        //         _ => unimplemented!("for now only Lookup constraint is implemented"),
        //     };
        //
        //     values.append(&mut matrix.values);
        //     final_width += matrix.width;
        // }

        let mut matrixes = vec![];
        let mut final_width = 0;

        for c in &self.constraints {
            let matrix = match c {
                // Constraint::Permutation(p) => p.get_trace(challange),
                Constraint::Lookup(l) => { l.get_trace(challange) },
                _ => unimplemented!("for now only Lookup constraint is implemented"),
            };

            final_width += matrix.width;
            matrixes.push(matrix);
        }

        // TODO: we need to define max height
        let mut max_height = matrixes[0].height();

        let mut values = vec![];

        for row in 0..max_height {
            for matrix in &matrixes {
                let rows = matrix.values.len() / matrix.width;
                if row < rows {
                    // Append the current row's data
                    let start = row * matrix.width;
                    let end = usize::min(start + matrix.width, matrix.values.len());
                    values.append(&mut matrix.values[start..end].to_vec());
                } else {
                    // If the current matrix has fewer rows, pad with zeros
                    values.extend(vec![Bls12_377Fr::ZERO; matrix.width]);
                }
            }
        }

        println!("final_width {}", final_width);

        return RowMajorMatrix::new(values, final_width);
    }

    pub fn get_air_configs(&self) -> Vec<AirConfig> {
        self.constraints.iter().map(|c| {
            match c {
                Constraint::Permutation(_) => unimplemented!("permutation constraint is not currently implemented"),
                Constraint::Lookup(l) => AirConfig::Lookup(AirLookupConfig::new(l.a_width(), l.b_width()))
            }
        }).collect()
    }
}

pub fn read_permutation(path: &str) -> RawPermutationTrace {
    let file_content = fs::read(path).unwrap();
    let raw_trace: RawPermutationTrace =
        ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();
    raw_trace
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawLookupTrace {
    pub a: Vec<Vec<Vec<u8>>>,
    pub b: Vec<Vec<Vec<u8>>>,
    pub name: String,
    pub a_filter: Vec<Vec<u8>>,
    pub b_filter: Vec<Vec<u8>>,
}

pub fn read_lookup(path: &str) -> RawLookupTrace {
    let file_content = fs::read(path).unwrap();
    let mut raw_trace: RawLookupTrace =
        ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

    raw_trace
}
