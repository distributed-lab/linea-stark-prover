use crate::range::RawRangeTrace;
use air::configs::AirLookupConfig;
use ark_ff::{BigInteger, PrimeField};
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fs;

type LookupColumn = Vec<Vec<Vec<Bls12_377Fr>>>;

pub struct LookupColumns {
    pub a: LookupColumn,
    pub b: LookupColumn,
    pub a_filter: Vec<Vec<Bls12_377Fr>>,
    pub b_filter: Vec<Vec<Bls12_377Fr>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawLookupTrace {
    pub a: Vec<Vec<Vec<[u8; 32]>>>,
    pub a_ids: Vec<Vec<String>>,
    pub b: Vec<Vec<Vec<[u8; 32]>>>,
    pub b_ids: Vec<Vec<String>>,
    pub name: String,
    pub a_filter: Vec<Vec<[u8; 32]>>,
    pub b_filter: Vec<Vec<[u8; 32]>>,
}

impl RawLookupTrace {
    pub fn read_file(path: &str) -> Result<RawLookupTrace, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawLookupTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();
        Ok(raw_trace)
    }

    pub fn is_a_filtered(&self) -> bool {
        !self.a_filter.is_empty() && !self.a_filter[0].is_empty()
    }

    pub fn is_b_filtered(&self) -> bool {
        // We have only two cases - all filters for B is included or no filters for B at all
        !self.b_filter.is_empty() && !self.b_filter[0].is_empty()
    }

    pub(crate) fn resize(&mut self, size: usize) {
        for a_element in &mut self.a {
            for e in a_element {
                e.resize(size, [0u8; 32]);
            }
        }

        if self.is_a_filtered() {
            // Resize only if B filters enabled
            for a_filter_row in &mut self.a_filter {
                a_filter_row.resize(size, [0u8; 32]);
            }
        }

        for b_element in &mut self.b {
            for e in b_element {
                e.resize(size, [0u8; 32]);
            }
        }

        if self.is_b_filtered() {
            // Resize only if B filters enabled
            for b_filter_row in &mut self.b_filter {
                b_filter_row.resize(size, [0u8; 32]);
            }
        }
    }

    pub(crate) fn set_columns(
        &mut self,
        mut a: LookupColumn,
        mut b: LookupColumn,
        mut a_filter: Vec<Vec<Bls12_377Fr>>,
        mut b_filter: Vec<Vec<Bls12_377Fr>>,
        columns: &mut [Vec<Bls12_377Fr>],
        cfg: &AirLookupConfig,
    ) -> LookupColumns {
        // Get a, b columns

        for a_table_id in 0..a.len() {
            for a_col_id in 0..a[a_table_id].len() {
                columns[cfg.a_columns_ids[a_table_id][a_col_id]] =
                    std::mem::take(&mut a[a_table_id][a_col_id]);
            }
        }

        for b_table_id in 0..b.len() {
            for b_col_id in 0..b[b_table_id].len() {
                columns[cfg.b_columns_ids[b_table_id][b_col_id]] =
                    std::mem::take(&mut b[b_table_id][b_col_id]);
            }
        }

        if let Some(a_filter_ids) = &cfg.a_filter_id {
            for (i, id) in a_filter_ids.iter().enumerate() {
                columns[*id] = std::mem::take(&mut a_filter[i]);
            }
        }

        if let Some(b_filter_ids) = &cfg.b_filter_id {
            for (i, id) in b_filter_ids.iter().enumerate() {
                columns[*id] = std::mem::take(&mut b_filter[i]);
            }
        }

        LookupColumns {
            a,
            b,
            a_filter,
            b_filter,
        }
    }

    pub(crate) fn set_trace(
        &mut self,
        challenges: Vec<Bls12_377Fr>,
        columns: &mut [Vec<Bls12_377Fr>],
        cfg: &AirLookupConfig,
    ) {
        assert_eq!(
            challenges.len(),
            2,
            "Two challenges should be provided for the lookup trace"
        );

        // Unpack challenges
        let (alpha, delta) = (challenges[0], challenges[1]);

        let (a, b) = self.get_columns();
        let a_filter = self.get_a_filters();
        let b_filter = self.get_b_filters();

        let a_filter_id = cfg.a_filter_id.as_ref();
        let b_filter_id = cfg.b_filter_id.as_ref();

        // Trace height
        // !IMPORTANT: should be equal per all columns.
        let sz = a[0][0].len();

        self.set_columns(a, b, a_filter, b_filter, columns, cfg);

        // Amount of occurrence pre unique row in A
        let mut occurrences: HashMap<Bls12_377Fr, usize> = HashMap::new();

        // Build occurrence mapping (should be done before trace generation)
        // TODO: this is a partially repeated piece of code. Think how write it better.
        for i in 0..sz {
            for (a_table_index, a_col_indexes) in cfg.a_columns_ids.iter().enumerate() {
                // Skip is disabled by filter
                if self.is_a_filtered() && columns[a_filter_id.unwrap()[a_table_index]][i].is_zero()
                {
                    continue;
                }

                let mut a_row_comb = Bls12_377Fr::ZERO;
                for a_id in a_col_indexes {
                    // Collect linear combination of the row
                    // `a_row_comb = a[i][j] * alpha^j` per all `j`
                    a_row_comb = a_row_comb * alpha + columns[*a_id][i];
                }

                // Update occurrences of the A row linear combination
                if let Some(count) = occurrences.get(&a_row_comb) {
                    occurrences.insert(a_row_comb, *count + 1);
                } else {
                    occurrences.insert(a_row_comb, 1);
                }
            }
        }

        let mut a_inverses_table: Vec<Vec<Bls12_377Fr>> =
            (0..cfg.a_inverses_id.len()).map(|_| Vec::new()).collect();

        let mut b_inverses_table: Vec<Vec<Bls12_377Fr>> =
            (0..cfg.b_inverses_id.len()).map(|_| Vec::new()).collect();

        let mut multiplicities_table: Vec<Vec<Bls12_377Fr>> =
            (0..cfg.b_inverses_id.len()).map(|_| Vec::new()).collect();

        let mut prefix_sum_column = Vec::new();

        // Total sum of the log-derivative terms with corresponding multiplicities:
        // one per each A row and -m[i] per each B row (m should be properly handled)
        let mut log_derivative_sum = Bls12_377Fr::ZERO;

        for i in 0..sz {
            for (a_table_index, a_col_indexes) in cfg.a_columns_ids.iter().enumerate() {
                let mut a_row_comb = Bls12_377Fr::ZERO;
                for a_id in a_col_indexes {
                    // Iterate over all A columns and collect linear combination of the row
                    // `a_row_comb = a[i][j] * alpha^j` per all `j`
                    a_row_comb = a_row_comb * alpha + columns[*a_id][i];
                }

                let a_row_comb_inverse = (a_row_comb + delta).inverse();
                a_inverses_table[a_table_index].push(a_row_comb_inverse);

                // If the current A row is not disabled by filter
                // (otherwise it is assumed to be multiplied on zero filter value)
                if !self.is_a_filtered() || columns[a_filter_id.unwrap()[a_table_index]][i].is_one()
                {
                    log_derivative_sum += a_row_comb_inverse;
                }
            }

            for (b_table_index, b_col_indexes) in cfg.b_columns_ids.iter().enumerate() {
                let mut b_row_comb = Bls12_377Fr::ZERO;
                for b_ind in b_col_indexes {
                    // Iterate over all B columns and collect linear combination of the row
                    // `b_row_comb = b[i][j] * alpha^j` per all `j`
                    b_row_comb = b_row_comb * alpha + columns[*b_ind][i];
                }

                let b_row_comb_inverse = (b_row_comb + delta).inverse();
                b_inverses_table[b_table_index].push(b_row_comb_inverse);

                let mut occurrence = Bls12_377Fr::ZERO;
                if let Some(cnt) = occurrences.get(&b_row_comb) {
                    let should_remove = if self.is_b_filtered() {
                        columns[b_filter_id.unwrap()[b_table_index]][i].is_one()
                    } else {
                        true
                    };

                    if should_remove {
                        occurrence = Bls12_377Fr::from_canonical_usize(*cnt);
                        log_derivative_sum -= b_row_comb_inverse * occurrence;
                        occurrences.remove(&b_row_comb);
                    }
                }

                multiplicities_table[b_table_index].push(occurrence);
            }

            prefix_sum_column.push(log_derivative_sum);
        }

        assert!(
            prefix_sum_column.last().unwrap().is_zero(),
            "failed to check constrain: check column should be 0 on the last row"
        );

        for (i, id) in cfg.a_inverses_id.iter().enumerate() {
            columns[*id] = std::mem::take(&mut a_inverses_table[i]);
        }

        for (i, id) in cfg.b_inverses_id.iter().enumerate() {
            columns[*id] = std::mem::take(&mut b_inverses_table[i].clone());
        }

        for (i, id) in cfg.occurrences_id.iter().enumerate() {
            columns[*id] = std::mem::take(&mut multiplicities_table[i].clone());
        }

        columns[cfg.check_id] = std::mem::take(&mut prefix_sum_column);
    }

    pub fn get_max_height(&self) -> usize {
        let mut max_height = 0_usize;

        self.a.iter().for_each(|ai| {
            ai.iter().for_each(|aij| {
                max_height = max(max_height, aij.len());
            })
        });

        self.b.iter().for_each(|bi| {
            bi.iter().for_each(|bij| {
                max_height = max(max_height, bij.len());
            })
        });

        max_height
    }

    pub fn get_columns(&mut self) -> (LookupColumn, LookupColumn) {
        let mut a: LookupColumn = Vec::new();
        let mut b: LookupColumn = Vec::new();

        for i in 0..self.a.len() {
            a.push(Vec::new());

            for j in 0..self.a[i].len() {
                a[i].push(Vec::new());
                for k in 0..self.a[i][j].len() {
                    a[i][j].push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                        self.a[i][j][k].as_slice(),
                    )));
                }
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

    pub fn get_a_filters(&mut self) -> Vec<Vec<Bls12_377Fr>> {
        let mut a_filter_field: Vec<Vec<Bls12_377Fr>> = vec![Vec::new(); self.a.len()];

        if !self.is_a_filtered() {
            return a_filter_field;
        }

        for (i, a_filter_row) in a_filter_field.iter_mut().enumerate().take(self.a.len()) {
            for a_filter_value in &self.a_filter[i] {
                a_filter_row.push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    a_filter_value.as_slice(),
                )));
            }
        }

        a_filter_field
    }

    pub fn get_b_filters(&mut self) -> Vec<Vec<Bls12_377Fr>> {
        let mut b_filter_field: Vec<Vec<Bls12_377Fr>> = vec![Vec::new(); self.b.len()];

        if !self.is_b_filtered() {
            return b_filter_field;
        }

        for (i, b_filter_row) in b_filter_field.iter_mut().enumerate().take(self.b.len()) {
            for b_filter_value in &self.b_filter[i] {
                b_filter_row.push(Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
                    b_filter_value.as_slice(),
                )));
            }
        }

        b_filter_field
    }

    pub fn update_registry(
        &self,
        columns_registry: &mut HashMap<String, usize>,
        columns: &mut Vec<Vec<Bls12_377Fr>>,
    ) -> AirLookupConfig {
        let mut next_id = || -> usize {
            columns.push(Vec::new());
            columns.len() - 1
        };

        let mut a_columns_ids = vec![Vec::<usize>::new(); self.a.len()];

        for (i, ids) in a_columns_ids.iter_mut().enumerate().take(self.a.len()) {
            for name in &self.a_ids[i] {
                if let Some(id) = columns_registry.get(name) {
                    ids.push(*id);
                } else {
                    let id = next_id();
                    columns_registry.insert(name.clone(), id);
                    ids.push(id);
                }
            }
        }

        let mut b_columns_ids = vec![Vec::<usize>::new(); self.b.len()];

        for (i, ids) in b_columns_ids.iter_mut().enumerate().take(self.b.len()) {
            for name in &self.b_ids[i] {
                if let Some(id) = columns_registry.get(name) {
                    ids.push(*id);
                } else {
                    let id = next_id();
                    columns_registry.insert(name.clone(), id);
                    ids.push(id);
                }
            }
        }

        let mut a_filter_id = None;
        if self.is_a_filtered() {
            a_filter_id = Some((0..self.a.len()).map(|_| next_id()).collect());
        }

        let mut b_filter_id = None;
        if self.is_b_filtered() {
            b_filter_id = Some((0..self.b.len()).map(|_| next_id()).collect());
        }

        let a_inverses_id: Vec<usize> = (0..self.a.len()).map(|_| next_id()).collect();
        let b_inverses_id: Vec<usize> = (0..self.b.len()).map(|_| next_id()).collect();
        let occurrences_id: Vec<usize> = (0..self.b.len()).map(|_| next_id()).collect();
        let check_id = next_id();

        AirLookupConfig {
            a_columns_ids,
            b_columns_ids,
            a_filter_id,
            b_filter_id,
            a_inverses_id,
            b_inverses_id,
            occurrences_id,
            check_id,
        }
    }
}

impl From<RawRangeTrace> for RawLookupTrace {
    fn from(value: RawRangeTrace) -> Self {
        let mut a: Vec<Vec<Vec<[u8; 32]>>> = vec![];
        let mut a_ids: Vec<Vec<String>> = vec![];
        let mut b: Vec<Vec<Vec<[u8; 32]>>> = vec![vec![Vec::new()]];

        for col_a in value.a {
            a.push(vec![col_a]);
        }

        for col_a_id in value.a_id {
            a_ids.push(vec![col_a_id]);
        }

        let mut counter = 0u64;

        while counter < value.b {
            b[0][0].push(
                Bls12_377Fr::from_canonical_u64(counter)
                    .value
                    .into_bigint()
                    .to_bytes_be()
                    .as_slice()
                    .try_into()
                    .unwrap(),
            );
            counter += 1;
        }

        Self {
            a,
            a_ids,
            b,
            b_ids: vec![vec![format!("{}", value.b)]],
            name: value.name,
            a_filter: vec![],
            b_filter: vec![],
        }
    }
}
