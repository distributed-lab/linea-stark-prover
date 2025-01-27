use air::air_lookup::AirLookupConfig;
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawLookupTrace {
    pub a: Vec<Vec<[u8; 32]>>,
    pub b: Vec<Vec<[u8; 32]>>,
    pub name: String,
    pub a_filter: Vec<[u8; 32]>,
    pub b_filter: Vec<[u8; 32]>,
}

impl RawLookupTrace {
    pub fn read_file(path: &str) -> Self {
        let file_content = fs::read(path).unwrap();
        let raw_trace: RawLookupTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        raw_trace
    }

    pub(crate) fn get_trace(
        &self,
        challenges: Vec<Bls12_377Fr>,
    ) -> (AirLookupConfig, Vec<Vec<Bls12_377Fr>>) {
        assert_eq!(
            challenges.len(),
            2,
            "Two challenges should be provided for the lookup trace"
        );

        // Unpack challenges
        let (alpha, delta) = (challenges[0], challenges[1]);

        // a columns, b columns, and corresponding filters
        let (a, b, a_filter, b_filter) = self.get_columns();

        // Resulting trace in one-dimensional array
        let mut res: Vec<Vec<Bls12_377Fr>> = Vec::new();

        res.append(&mut a.clone());
        res.append(&mut b.clone());
        res.push(a_filter.clone());
        res.push(b_filter.clone());

        // Trace height
        // !IMPORTANT: should be equal per all columns.
        let sz = a[0].len();

        // Amount of occurrence pre unique row in A
        let mut occurrences: HashMap<Bls12_377Fr, usize> = HashMap::new();

        // Build occurrence mapping (should be done before trace generation)
        // TODO: this is a partially repeated piece of code. Think how write it better.
        for i in 0..sz {
            // Skip is disabled by filter
            if a_filter[i] == Bls12_377Fr::ZERO {
                continue;
            }

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
        let mut b_inverses_column = Vec::new();
        let mut multiplicities_column = Vec::new();
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

            // If the current A row is not disabled by filter
            // (otherwise it is assumed to be multiplied on zero filter value)
            if a_filter[i] != Bls12_377Fr::ZERO {
                // Add A row log-derivative term
                log_derivative_sum += a_row_comb_inverse
            }

            let mut b_row_comb = Bls12_377Fr::ZERO;

            for b_column in &b {
                // Iterate over all B columns and collect linear combination of the row
                // `b_row_comb = b[i][j] * alpha^j` per all `j`
                b_row_comb = b_row_comb * alpha + b_column[i];
            }

            let b_row_comb_inverse = (b_row_comb + delta).inverse();
            b_inverses_column.push(b_row_comb_inverse);

            // Get multiplicity for current B row
            let mut occurrence = Bls12_377Fr::ZERO;
            if let Some(cnt) = occurrences.get(&b_row_comb) {
                if b_filter[i] != Bls12_377Fr::ZERO {
                    // If multiplicity is non-zero and B row is not disabled by filter, then:
                    // - subtract from sum the corresponding log-derivative term
                    // - remove multiplicity from occurrences
                    occurrence = Bls12_377Fr::from_canonical_usize(*cnt);
                    log_derivative_sum -= b_row_comb_inverse * occurrence;
                    occurrences.remove(&b_row_comb);
                }
            }

            multiplicities_column.push(occurrence);
            prefix_sum_column.push(log_derivative_sum);
        }

        assert!(
            prefix_sum_column.last().unwrap().is_zero(),
            "failed to check constrain: check column should be 0 on the last row"
        );

        res.push(a_inverses_column);
        res.push(b_inverses_column);
        res.push(multiplicities_column);
        res.push(prefix_sum_column);

        (
            AirLookupConfig {
                a_columns_ids: (0..a.len()).collect(),
                b_columns_ids: (a.len()..a.len() + b.len()).collect(),
                a_filter_id: a.len() + b.len(),
                b_filter_id: a.len() + b.len() + 1,
                a_inverses_id: a.len() + b.len() + 2,
                b_inverses_id: a.len() + b.len() + 3,
                occurrences_id: a.len() + b.len() + 4,
                check_id: a.len() + b.len() + 5,
            },
            res,
        )
    }

    pub fn resize(&mut self, size: usize) {
        for e in &mut self.a {
            e.resize(size, [0u8; 32]);
        }

        self.a_filter.resize(size, [0u8; 32]);

        for e in &mut self.b {
            e.resize(size, [0u8; 32]);
        }

        self.b_filter.resize(size, [0u8; 32]);
    }

    pub fn get_columns(
        &self,
    ) -> (
        Vec<Vec<Bls12_377Fr>>,
        Vec<Vec<Bls12_377Fr>>,
        Vec<Bls12_377Fr>,
        Vec<Bls12_377Fr>,
    ) {
        let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
        let mut b: Vec<Vec<Bls12_377Fr>> = Vec::new();

        let mut a_filter: Vec<Bls12_377Fr> = Vec::new();
        let mut b_filter: Vec<Bls12_377Fr> = Vec::new();

        for i in 0..self.a.len() {
            a.push(Vec::new());
            for j in 0..self.a[i].len() {
                a[i].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.a[i][j].as_slice(),
                )));
            }
        }

        for i in 0..self.a_filter.len() {
            a_filter.push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                self.a_filter[i].as_slice(),
            )))
        }

        for i in 0..self.b.len() {
            b.push(Vec::new());
            for j in 0..self.b[i].len() {
                b[i].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.b[i][j].as_slice(),
                )));
            }
        }

        for i in 0..self.b_filter.len() {
            b_filter.push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                self.b_filter[i].as_slice(),
            )))
        }

        (a, b, a_filter, b_filter)
    }
}
