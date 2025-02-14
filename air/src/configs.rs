#[derive(Clone, Debug)]
pub struct AirLookupConfig {
    pub a_columns_ids: Vec<Vec<usize>>,
    pub b_columns_ids: Vec<Vec<usize>>,
    pub a_filter_id: Option<Vec<usize>>,
    pub b_filter_id: Option<Vec<usize>>,
    pub a_inverses_id: Vec<usize>,
    pub b_inverses_id: Vec<usize>,
    pub occurrences_id: Vec<usize>,
    pub check_id: usize,
}

#[derive(Clone, Debug)]
pub struct AirPermutationConfig {
    pub a_columns_ids: Vec<usize>,
    pub b_columns_ids: Vec<usize>,
    pub b_inverse_id: usize,
    pub check_id: usize,
}