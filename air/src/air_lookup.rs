use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues};
use p3_field::FieldAlgebra;
use p3_matrix::Matrix;

#[derive(Clone, Debug)]
pub struct AirLookupConfig {
    /// 0...a-1 it is the A columns
    pub a_shift: usize,
    /// a...2a-1 it is the A\[i]+challenge inverse
    pub a_inv_shift: usize,
    pub a_width: usize,
    /// 2a...2a+b-1 it is the B columns
    pub b_shift: usize,
    /// 2a+b...2a+2b-1 it is the B\[i]+challenge inverse
    pub b_inv_shift: usize,
    pub b_width: usize,
    /// 2a+2b ...2a+3b-1 it is the occurrences of B\[i] in A
    pub occurrences_column_shift: usize,
    /// 2a+3b it is a check lookup constrain column
    pub check_column_shift: usize,
    pub a_filter_column_shift: usize,
    pub b_filter_column_shift: usize,
    /// constraint matrix width
    pub width: usize,
}

impl AirLookupConfig {
    pub fn new(a_width: usize, b_width: usize) -> Self {
        AirLookupConfig {
            a_shift: 0,
            a_inv_shift: a_width,
            a_width,
            b_shift: a_width * 2,
            b_inv_shift: a_width * 2 + b_width,
            b_width,
            occurrences_column_shift: a_width * 2 + 2 * b_width,
            check_column_shift: a_width * 2 + 3 * b_width,
            a_filter_column_shift: unimplemented!("implement a_filter_column_shift"),
            b_filter_column_shift: unimplemented!("implement b_filter_column_shift"),
            width: a_width * 2 + 3 * b_width + 1,
        }
    }
}
