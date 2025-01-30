use crate::lookup::RawLookupTrace;
use air::air_lookup::AirLookupConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fs;

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

        let b = self.get_raw_b_column();

        let mut lookup_raw_trace = RawLookupTrace {
            a: vec![self.a],
            b: vec![vec![b]],
            name: self.name.clone(),
            a_filter: vec![],
            b_filter: vec![],
        };

        lookup_raw_trace.resize(self.a.len());
        lookup_raw_trace.get_trace(challenges)
    }

    pub fn get_max_height(&self) -> usize {
        max(self.a.len(), self.b as usize)
    }

    pub fn resize(&mut self, size: usize) {
        self.a.resize(size, [0u8; 32]);
    }

    pub fn get_raw_b_column(&mut self) -> (Vec<[u8; 32]>) {
        let mut b: Vec<[u8; 32]> = Vec::new();
        let mut counter = 0u64;

        for _ in 0..self.a.len() {
            let mut buf = [0u8; 32];
            if counter < self.b {
                let bytes = counter.to_be_bytes();
                buf[24..].copy_from_slice(&bytes);
                b.push(buf);
                counter += 1;
            } else {
                b.push(buf);
            }
        }

        b
    }
}
