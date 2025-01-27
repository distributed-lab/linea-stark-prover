use air::air_permutation::AirPermutationConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawPermutationTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub b: Vec<Vec<[u8; 32]>>,
    pub name: String,
}

impl RawPermutationTrace {
    pub fn read_file(path: &str) -> Self {
        let file_content = fs::read(path).unwrap();
        let raw_trace: RawPermutationTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();
        raw_trace
    }

    pub fn get_trace(
        &self,
        challenges: Vec<Bls12_377Fr>,
    ) -> (AirPermutationConfig, Vec<Vec<Bls12_377Fr>>) {
        assert_eq!(
            challenges.len(),
            2,
            "Two challenges should be provided for the lookup trace"
        );

        // Unpack challenges
        let (alpha, delta) = (challenges[0], challenges[1]);

        // a columns, b columns
        let (mut a, mut b) = self.get_columns();

        let sz = a[0].len();

        let width = a.len();

        let mut res: Vec<Vec<Bls12_377Fr>> = Vec::new();

        res.append(&mut a.clone());
        res.append(&mut b.clone());

        // Prefix multiplication of the permutation terms
        let mut prev_check = Bls12_377Fr::ONE;

        let mut b_inverse_column = Vec::new();
        let mut perm_check_column = Vec::new();

        for i in 0..sz {
            let mut a_row_comb = Bls12_377Fr::ZERO;
            for a_column in &a {
                // Collect linear combination of the row
                // `a_row_comb = a[i][j] * alpha^j` per all `j`
                a_row_comb = a_row_comb * alpha + a_column[i];
            }

            let mut b_row_comb = Bls12_377Fr::ZERO;
            for b_column in &b {
                // Iterate over all B columns and collect linear combination of the row
                // `b_row_comb = b[i][j] * alpha^j` per all `j`
                b_row_comb = b_row_comb * alpha + b_column[i];
            }

            let b_row_comb_inverse = (b_row_comb + delta).inverse();
            b_inverse_column.push(b_row_comb_inverse);
            prev_check = prev_check * (a_row_comb + delta) * b_row_comb_inverse;
            perm_check_column.push(prev_check);
        }

        assert!(
            perm_check_column.last().unwrap().is_one(),
            "failed to check constrain: check column should be 1 on the last row"
        );

        res.push(b_inverse_column);
        res.push(perm_check_column);

        (
            AirPermutationConfig {
                a_columns_ids: (0..width).collect(),
                b_columns_ids: (width..2 * width).collect(),
                b_inverse_id: 2 * width,
                check_id: 2 * width + 1,
            },
            res,
        )
    }

    pub fn get_columns(&self) -> (Vec<Vec<Bls12_377Fr>>, Vec<Vec<Bls12_377Fr>>) {
        let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
        let mut b: Vec<Vec<Bls12_377Fr>> = Vec::new();

        for i in 0..self.a.len() {
            a.push(Vec::new());
            for j in 0..self.a[i].len() {
                a[i].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.a[i][j].as_slice(),
                )));
            }
        }

        for i in 0..self.b.len() {
            b.push(Vec::new());
            for j in 0..self.b[i].len() {
                b[i].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.b[i][j].as_slice(),
                )));
            }
        }

        (a, b)
    }

    pub fn resize(&mut self, size: usize) {
        for e in &mut self.a {
            e.resize(size, [0u8; 32]);
        }

        for e in &mut self.b {
            e.resize(size, [0u8; 32]);
        }
    }
}
