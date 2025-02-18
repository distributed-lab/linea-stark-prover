use air::configs::AirPermutationConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawPermutationTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub a_ids: Vec<String>,
    pub b: Vec<Vec<[u8; 32]>>,
    pub b_ids: Vec<String>,
    pub name: String,
}

impl RawPermutationTrace {
    pub fn read_file(path: &str) -> Result<Self, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawPermutationTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();
        Ok(raw_trace)
    }

    pub(crate) fn resize(&mut self, size: usize) {
        for e in &mut self.a {
            e.resize(size, [0u8; 32]);
        }

        for e in &mut self.b {
            e.resize(size, [0u8; 32]);
        }
    }

    pub fn set_trace(
        &self,
        challenges: Vec<Bls12_377Fr>,
        columns: &mut [Vec<Bls12_377Fr>],
        cfg: &AirPermutationConfig,
    ) {
        assert_eq!(
            challenges.len(),
            2,
            "Two challenges should be provided for the lookup trace"
        );

        // Unpack challenges
        let (alpha, delta) = (challenges[0], challenges[1]);

        // a columns, b columns
        let (a, b) = self.get_columns();

        for (i, id) in cfg.a_columns_ids.iter().enumerate() {
            columns[*id] = a[i].clone();
        }

        for (i, id) in cfg.b_columns_ids.iter().enumerate() {
            columns[*id] = b[i].clone();
        }

        // Prefix multiplication of the permutation terms
        let mut prev_check = Bls12_377Fr::ONE;

        let mut b_inverse_column = Vec::new();
        let mut perm_check_column = Vec::new();

        let sz = a[0].len();

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

        columns[cfg.b_inverse_id] = b_inverse_column;
        columns[cfg.check_id] = perm_check_column;
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

    pub fn update_registry(
        &self,
        columns_registry: &mut HashMap<String, usize>,
        columns: &mut Vec<Vec<Bls12_377Fr>>,
    ) -> AirPermutationConfig {
        let mut a_columns_ids = Vec::new();

        let mut next_id = || -> usize {
            columns.push(Vec::new());
            columns.len() - 1
        };

        for name in &self.a_ids {
            if let Some(id) = columns_registry.get(name) {
                a_columns_ids.push(*id);
            } else {
                let id = next_id();
                columns_registry.insert(name.clone(), id);
                a_columns_ids.push(id);
            }
        }

        let mut b_columns_ids = Vec::new();

        for name in &self.b_ids {
            if let Some(id) = columns_registry.get(name) {
                b_columns_ids.push(*id);
            } else {
                let id = next_id();
                columns_registry.insert(name.clone(), id);
                b_columns_ids.push(id);
            }
        }

        let b_inverse_id = next_id();
        let check_id = next_id();

        AirPermutationConfig {
            a_columns_ids,
            b_columns_ids,
            b_inverse_id,
            check_id,
        }
    }

    pub fn get_max_height(&self) -> usize {
        let mut max_height = 0_usize;
        self.a.iter().for_each(|ai| {
            max_height = max(max_height, ai.len());
        });

        self.b.iter().for_each(|bi| {
            bi.iter().for_each(|bij| {
                max_height = max(max_height, bij.len());
            })
        });

        max_height
    }
}
