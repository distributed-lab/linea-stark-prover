#[derive(Clone, Debug)]
pub struct AirLookupNoFiltersConfig {
    pub a_columns_ids: Vec<usize>,
    pub b_columns_ids: Vec<Vec<usize>>,
    pub a_inverses_id: usize,
    pub b_inverses_id: Vec<usize>,
    pub occurrences_id: Vec<usize>,
    pub check_id: usize,
}

impl AirLookupNoFiltersConfig {
    pub fn width(&self) -> usize {
        self.a_columns_ids.len() + self.b_columns_ids.len() * (self.b_columns_ids[0].len() + 2) + 2
    }
}
