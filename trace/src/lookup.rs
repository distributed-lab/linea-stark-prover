use std::collections::HashMap;
use std::fs;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use serde::{Deserialize, Serialize};

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
            // let mut a_inverses = Vec::new();

            for col in &a {
                let aji_inv = (col[i] + challenge).inverse();
                sum += aji_inv;
                // a_inverses.push(aji_inv);
                res.push(col[i]);
            }

            // res.append(&mut a_inverses);

            // let mut b_inverses = Vec::new();
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
                // b_inverses.push(bji_inv);
                res.push(col[i]);
            }

            // res.append(&mut b_inverses);
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
