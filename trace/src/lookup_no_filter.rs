use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawLookupNoFilterTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub a_ids: Vec<String>,
    pub b: Vec<Vec<Vec<[u8; 32]>>>,
    pub b_ids: Vec<Vec<String>>,
    pub name: String,
}

impl RawLookupNoFilterTrace {
    pub fn read_file(path: &str) -> Result<RawLookupNoFilterTrace, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawLookupNoFilterTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        Ok(raw_trace)
    }

    pub fn resize(&mut self, size: usize) {
        for e in &mut self.a {
            e.resize(size, [0u8; 32]);
        }
        
        for b_element in &mut self.b {
            for e in b_element {
                e.resize(size, [0u8; 32]);
            }
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
    
    pub fn get_columns(&mut self) -> (Vec<Vec<Bls12_377Fr>>, Vec<Vec<Vec<Bls12_377Fr>>>) {
        let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
        let mut b: Vec<Vec<Vec<Bls12_377Fr>>> = Vec::new();

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
                b[i].push(Vec::new());
                for k in 0..self.b[i][j].len() {
                    b[i][j].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                        self.b[i][j][k].as_slice(),
                    )));
                }
            }
        }

        (a, b)
    }
}
