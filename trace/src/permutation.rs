use std::fs;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use serde::{Deserialize, Serialize};

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