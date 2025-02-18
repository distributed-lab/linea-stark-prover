use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawRangeTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub a_id: Vec<String>,
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
        let mut height = self.b as usize;
        for i in 0..self.a.len() {
            height = max(height, self.a[i].len())
        }
        height
    }
}
