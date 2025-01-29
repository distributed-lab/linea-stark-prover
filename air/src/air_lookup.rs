#[derive(Clone, Debug)]
pub struct AirLookupConfig {
    pub a_columns_ids: Vec<usize>,
    pub b_columns_ids: Vec<Vec<usize>>,
    pub a_filter_id: usize,
    pub b_filter_id: Vec<usize>,
    pub a_inverses_id: usize,
    pub b_inverses_id: Vec<usize>,
    pub occurrences_id: Vec<usize>,
    pub check_id: Vec<usize>,
}

impl AirLookupConfig {
    pub fn shift(&mut self, shift: usize) {
        self.a_columns_ids
            .iter_mut()
            .for_each(|i_column| *i_column = *i_column + shift);
        self.b_columns_ids.iter_mut().for_each(|i_table| {
            i_table
                .iter_mut()
                .for_each(|i_column| *i_column = *i_column + shift)
        });
        self.a_filter_id += shift;
        self.b_filter_id
            .iter_mut()
            .for_each(|i_column| *i_column = *i_column + shift);
        self.a_inverses_id += shift;
        self.b_inverses_id
            .iter_mut()
            .for_each(|i_column| *i_column = *i_column + shift);
        self.occurrences_id
            .iter_mut()
            .for_each(|i_column| *i_column = *i_column + shift);
        self.check_id
            .iter_mut()
            .for_each(|i_column| *i_column = *i_column + shift);
    }

    pub fn width(&self) -> usize {
        self.a_columns_ids.len() + self.b_columns_ids.len() * (self.b_columns_ids[0].len() + 4) + 2
    }
}
