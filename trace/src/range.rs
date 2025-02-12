use crate::lookup::{LookupColumns, RawLookupTrace};
use air::configs::AirLookupConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::FieldAlgebra;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;

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
}
