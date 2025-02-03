use crate::lookup::RawLookupTrace;
use air::air_lookup::AirLookupConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;
use p3_field::FieldAlgebra;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawRangeTrace {
    pub a: Vec<[u8; 32]>,
    pub b: u64,
    pub name: String,
}

impl RawRangeTrace {
    pub fn read_file(path: &str) -> Self {
        let file_content = fs::read(path).unwrap();
        let raw_trace: RawRangeTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        raw_trace
    }

    pub(crate) fn get_trace(
        &mut self,
        challenges: Vec<Bls12_377Fr>,
    ) -> (AirLookupConfig, Vec<Vec<Bls12_377Fr>>) {
        assert_eq!(
            challenges.len(),
            2,
            "Two challenges should be provided for the range trace"
        );
        // a columns, b columns, and corresponding filters
        let (a, b, a_filter, b_filter) = self.get_columns();

        RawLookupTrace::get_trace_from_columns(challenges, a, b, a_filter, b_filter)
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
