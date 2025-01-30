#[derive(Clone, Debug)]
pub struct AirPermutationConfig {
    pub a_columns_ids: Vec<usize>,
    pub b_columns_ids: Vec<usize>,
    pub b_inverse_id: usize,
    pub check_id: usize,
}

impl AirPermutationConfig {
    pub fn shift(&mut self, shift: usize) {
        self.a_columns_ids
            .iter_mut()
            .for_each(|i_column| *i_column = *i_column + shift);
        self.b_columns_ids
            .iter_mut()
            .for_each(|i_column| *i_column = *i_column + shift);
        self.b_inverse_id += shift;
        self.check_id += shift;
    }

    pub fn width(&self) -> usize {
        self.a_columns_ids.len() + self.b_columns_ids.len() + 2
    }
}
