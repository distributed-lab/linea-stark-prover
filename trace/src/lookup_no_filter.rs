use air::air_lookup_no_filter::AirLookupNoFiltersConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fs;
use std::process::id;

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

    pub(crate) fn set_trace(
        &mut self,
        challenges: Vec<Bls12_377Fr>,
        columns: &mut Vec<Vec<Bls12_377Fr>>,
        cfg: &AirLookupNoFiltersConfig,
    ) {
        assert_eq!(
            challenges.len(),
            2,
            "Two challenges should be provided for the lookup trace"
        );

        // Unpack challenges
        let (alpha, delta) = (challenges[0], challenges[1]);

        // Get a, b columns
        let (a, b) = self.get_columns();

        for (i, id) in cfg.a_columns_ids.iter().enumerate() {
            columns[*id] = a[i].clone();
        }

        for i in 0..cfg.b_columns_ids.len() {
            for (j, id) in cfg.b_columns_ids[i].iter().enumerate() {
                columns[*id] = b[i][j].clone();
            }
        }

        // Trace height
        // !IMPORTANT: should be equal per all columns.
        let sz = a[0].len();

        // Amount of occurrence pre unique row in A
        let mut occurrences: HashMap<Bls12_377Fr, usize> = HashMap::new();

        // Build occurrence mapping (should be done before trace generation)
        // TODO: this is a partially repeated piece of code. Think how write it better.
        for i in 0..sz {
            let mut a_row_comb = Bls12_377Fr::ZERO;
            for a_column in &a {
                // Collect linear combination of the row
                // `a_row_comb = a[i][j] * alpha^j` per all `j`
                a_row_comb = a_row_comb * alpha + a_column[i];
            }

            // Update occurrences of the A row linear combination
            if let Some(count) = occurrences.get(&a_row_comb) {
                occurrences.insert(a_row_comb, *count + 1);
            } else {
                occurrences.insert(a_row_comb, 1);
            }
        }

        let mut a_inverses_column = Vec::new();

        let mut b_inverses_table: Vec<Vec<Bls12_377Fr>> =
            (0..b.len()).map(|_| Vec::new()).collect();

        let mut multiplicities_table: Vec<Vec<Bls12_377Fr>> =
            (0..b.len()).map(|_| Vec::new()).collect();

        let mut prefix_sum_column = Vec::new();

        // Total sum of the log-derivative terms with corresponding multiplicities:
        // one per each A row and -m[i] per each B row (m should be properly handled)
        let mut log_derivative_sum = Bls12_377Fr::ZERO;

        for i in 0..sz {
            let mut a_row_comb = Bls12_377Fr::ZERO;
            for a_column in &a {
                // Iterate over all A columns and collect linear combination of the row
                // `a_row_comb = a[i][j] * alpha^j` per all `j`
                a_row_comb = a_row_comb * alpha + a_column[i];
            }

            let a_row_comb_inverse = (a_row_comb + delta).inverse();
            a_inverses_column.push(a_row_comb_inverse);

            // Add A row log-derivative term
            log_derivative_sum += a_row_comb_inverse;

            for (b_table_ind, b_table) in b.iter().enumerate() {
                let mut b_row_comb = Bls12_377Fr::ZERO;
                for b_column in b_table {
                    // Iterate over all B columns and collect linear combination of the row
                    // `b_row_comb = b[i][j] * alpha^j` per all `j`
                    b_row_comb = b_row_comb * alpha + b_column[i];
                }

                let b_row_comb_inverse = (b_row_comb + delta).inverse();
                b_inverses_table[b_table_ind].push(b_row_comb_inverse);

                let mut occurrence = Bls12_377Fr::ZERO;
                if let Some(cnt) = occurrences.get(&b_row_comb) {
                    // If multiplicity is non-zero, then:
                    // - subtract from sum the corresponding log-derivative term
                    // - remove multiplicity from occurrences
                    occurrence = Bls12_377Fr::from_canonical_usize(*cnt);
                    log_derivative_sum -= b_row_comb_inverse * occurrence;
                    occurrences.remove(&b_row_comb);
                }

                multiplicities_table[b_table_ind].push(occurrence);
            }

            prefix_sum_column.push(log_derivative_sum);
        }

        assert!(
            prefix_sum_column.last().unwrap().is_zero(),
            "failed to check constrain: check column should be 0 on the last row"
        );

        columns[cfg.a_inverses_id] = a_inverses_column;

        for (i, id) in cfg.b_inverses_id.iter().enumerate() {
            columns[*id] = b_inverses_table[i].clone();
        }

        for (i, id) in cfg.occurrences_id.iter().enumerate() {
            columns[*id] = multiplicities_table[i].clone();
        }

        columns[cfg.check_id] = prefix_sum_column.clone();
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

    pub fn update_registry(
        &self,
        columns_registry: &mut HashMap<String, usize>,
        columns: &mut Vec<Vec<Bls12_377Fr>>,
    ) -> AirLookupNoFiltersConfig {
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

        let mut b_columns_ids = vec![Vec::<usize>::new(); self.b.len()];

        for i in 0..self.b.len() {
            for name in &self.b_ids[i] {
                if let Some(id) = columns_registry.get(name) {
                    b_columns_ids[i].push(*id);
                } else {
                    let id = next_id();
                    columns_registry.insert(name.clone(), id);
                    b_columns_ids[i].push(id);
                }
            }
        }

        let a_inverses_id = next_id();
        let b_inverses_id: Vec<usize> = (0..self.b.len()).map(|_| next_id()).collect();
        let occurrences_id: Vec<usize> = (0..self.b.len()).map(|i| next_id()).collect();
        let check_id = next_id();

        AirLookupNoFiltersConfig {
            a_columns_ids,
            b_columns_ids,
            a_inverses_id,
            b_inverses_id,
            occurrences_id,
            check_id,
        }
    }
}
