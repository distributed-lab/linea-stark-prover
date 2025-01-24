use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;

#[derive(Clone, Debug)]
pub struct AirPermutationConfig {
    // Tables has the same sizes
    pub width: usize,
    // width * 2 + 1
    pub inv_column_index: usize,
    // width * 2
    pub check_column_index: usize,
}

impl AirPermutationConfig {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            check_column_index: width * 2,
            inv_column_index: width * 2 + 1,
        }
    }

    pub fn get_width(&self) -> usize {
        self.width * 2 + 2
    }
}

/// Processes the permutation for two columns
pub struct LineaPermutationAIR {
    // Tables has the same sizes
    pub width: usize,
    // width * 2 + 1
    pub inv_column_index: usize,
    // width * 2
    pub check_column_index: usize,
    // pub challenge: F,
}

/// | 0.       | 1.       | 2.                     | 3.                         |
/// |`Column A`|`Column B`|`Constrain check column`|`B+challenge inverse column`|

impl<F: Field> BaseAir<F> for LineaPermutationAIR {
    fn width(&self) -> usize {
        // a cols + b cols + check col + inv col
        self.width * 2 + 2
    }
}
