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
    pub b: Vec<Vec<Vec<[u8; 32]>>>,
    pub name: String,
    pub a_filter: Vec<[u8; 32]>,
    pub b_filter: Vec<Vec<[u8; 32]>>,
}

impl RawLookupTrace {
    pub fn read_file(path: &str) -> Self {
        let file_content = fs::read(path).unwrap();
        let mut raw_trace: RawLookupTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();

        // We have to append filters (enabled) in case of filters have been passed empty
        let mut one = [0u8; 32];
        one[31] = 1;

        while raw_trace.a_filter.len() < raw_trace.a[0].len() {
            raw_trace.a_filter.push(one);
        }

        while raw_trace.b_filter.len() < raw_trace.b.len() {
            raw_trace.b_filter.push(Vec::new());
        }

        for (b_filter_ind, b_filter) in raw_trace.b_filter.iter_mut().enumerate() {
            while b_filter.len() < raw_trace.b[b_filter_ind][0].len() {
                b_filter.push(one);
            }
        }

        raw_trace
    }

    pub(crate) fn get_trace(
        &mut self,
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
        let (a, mut b, a_filter, b_filter) = self.get_columns();

        // Resulting trace in one-dimensional array
        let mut res: Vec<Vec<Bls12_377Fr>> = Vec::new();

        res.append(&mut a.clone());

        for b_element in b.iter() {
            res.append(&mut b_element.clone());
        }

        res.push(a_filter.clone());
        res.append(&mut b_filter.clone());

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

            // If the current A row is not disabled by filter
            // (otherwise it is assumed to be multiplied on zero filter value)
            if a_filter[i] != Bls12_377Fr::ZERO {
                // Add A row log-derivative term
                log_derivative_sum += a_row_comb_inverse
            }

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
                    if b_filter[b_table_ind][i] != Bls12_377Fr::ZERO {
                        // If multiplicity is non-zero and B row is not disabled by filter, then:
                        // - subtract from sum the corresponding log-derivative term
                        // - remove multiplicity from occurrences
                        occurrence = Bls12_377Fr::from_canonical_usize(*cnt);
                        log_derivative_sum -= b_row_comb_inverse * occurrence;
                        occurrences.remove(&b_row_comb);
                    }
                }

                multiplicities_table[b_table_ind].push(occurrence);
                prefix_sum_column.push(log_derivative_sum);
            }
        }

        assert!(
            prefix_sum_column.last().unwrap().is_zero(),
            "failed to check constrain: check column should be 0 on the last row"
        );

        res.push(a_inverses_column);
        res.append(&mut b_inverses_table);
        res.append(&mut multiplicities_table);
        res.push(prefix_sum_column);

        let a_columns_ids = (0..a.len()).collect();

        let mut b_columns_ids: Vec<Vec<usize>> = (0..b.len()).map(|_| Vec::new()).collect();
        for i in 0..b.len() * b[0].len() {
            b_columns_ids[i / b[0].len()].push(i + a.len());
        }

        let a_filter_id = *b_columns_ids.last().unwrap().last().unwrap() + 1;

        let b_filter_id: Vec<usize> = (0..b.len()).map(|i| i + 1 + a_filter_id).collect();

        let a_inverses_id = b_filter_id.last().unwrap() + 1;

        let b_inverses_id: Vec<usize> = (0..b.len()).map(|i| i + 1 + a_inverses_id).collect();

        let occurrences_id: Vec<usize> = (0..b.len())
            .map(|i| i + 1 + b_inverses_id.last().unwrap())
            .collect();

        let check_id = occurrences_id.last().unwrap() + 1;

        (
            AirLookupConfig {
                a_columns_ids,
                b_columns_ids,
                a_filter_id,
                b_filter_id,
                a_inverses_id,
                b_inverses_id,
                occurrences_id,
                check_id,
            },
            res,
        )
    }

    pub fn resize(&mut self, size: usize) {
        for e in &mut self.a {
            e.resize(size, [0u8; 32]);
        }

        self.a_filter.resize(size, [0u8; 32]);

        for b_element in &mut self.b {
            for e in b_element {
                e.resize(size, [0u8; 32]);
            }
        }

        for b_filter in &mut self.b_filter {
            b_filter.resize(size, [0u8; 32]);
        }
    }

    pub fn get_columns(
        &mut self,
    ) -> (
        Vec<Vec<Bls12_377Fr>>,
        Vec<Vec<Vec<Bls12_377Fr>>>,
        Vec<Bls12_377Fr>,
        Vec<Vec<Bls12_377Fr>>,
    ) {
        let mut a: Vec<Vec<Bls12_377Fr>> = Vec::new();
        let mut b: Vec<Vec<Vec<Bls12_377Fr>>> = Vec::new();

        let mut a_filter: Vec<Bls12_377Fr> = Vec::new();
        let mut b_filter: Vec<Vec<Bls12_377Fr>> = Vec::new();

        for i in 0..self.a.len() {
            a.push(Vec::new());
            for j in 0..self.a[i].len() {
                a[i].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.a[i][j].as_slice(),
                )));

                a_filter.push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    self.a_filter[i].as_slice(),
                )));
            }
        }

        for i in 0..self.b.len() {
            b.push(Vec::new());

            b_filter.push(Vec::new());
            for j in 0..self.b[i].len() {
                b[i].push(Vec::new());
                for k in 0..self.b[i][j].len() {
                    b[i][j].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                        self.b[i][j][k].as_slice(),
                    )));

                    b_filter[i].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                        self.b_filter[i][j].as_slice(),
                    )));
                }
            }
        }

        (a, b, a_filter, b_filter)
    }
}
