use crate::lookup::{LookupColumns, RawLookupTrace};
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;
use p3_field::FieldAlgebra;
use air::configs::AirLookupConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawRangeTrace {
    pub a: Vec<[u8; 32]>,
    pub a_id: String,
    pub b: u64,
    pub name: String,
}

impl RawRangeTrace {
    pub fn read_file(path: &str) -> Result<RawRangeTrace, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawRangeTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        Ok(raw_trace)
    }

    pub fn get_max_height(&self) -> usize {
        max(self.a.len(), self.b as usize)
    }

    pub(crate) fn resize(&mut self, size: usize) {
        self.a.resize(size, [0u8; 32]);
    }

    fn get_columns(
        &mut self,
    ) -> (
        Vec<Vec<Bls12_377Fr>>,
        Vec<Vec<Vec<Bls12_377Fr>>>,
        Vec<Bls12_377Fr>,
        Vec<Vec<Bls12_377Fr>>,
    ) {
        let mut a: Vec<Vec<Bls12_377Fr>> = vec![Vec::new()];
        let mut b: Vec<Vec<Vec<Bls12_377Fr>>> = vec![vec![Vec::new()]];

        let mut a_filter: Vec<Bls12_377Fr> = Vec::new();
        let mut b_filter: Vec<Vec<Bls12_377Fr>> = vec![Vec::new()];

        for i in 0..self.a.len() {
            a[0].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                self.a[i].as_slice(),
            )));

            a_filter.push(Bls12_377Fr::ONE);
        }

        let mut counter = 0u64;

        for _ in 0..self.a.len() {
            if counter < self.b {
                b[0][0].push(Bls12_377Fr::from_canonical_u64(counter));
                counter += 1;
            } else {
                b[0][0].push(Bls12_377Fr::ZERO);
            }

            b_filter[0].push(Bls12_377Fr::ONE);
        }


        (a, b, a_filter, b_filter)
    }
}
