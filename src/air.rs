use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;

/// Processes the permutation for two columns
pub struct LineaPermutationAIR {}

/// | 0.       | 1.       | 2.                     | 3.                         |
/// |`Column A`|`Column B`|`Constrain check column`|`B+challenge inverse column`|
pub(crate) const PERMUTATION_WIDTH: usize = 4;
const PERMUTATION_COLUMN_A_INDEX: usize = 0;
const PERMUTATION_COLUMN_B_INDEX: usize = 1;
const PERMUTATION_COLUMN_CHECK_INDEX: usize = 2;
const PERMUTATION_COLUMN_INV_INDEX: usize = 3;

impl<F: Field> BaseAir<F> for LineaPermutationAIR {
    fn width(&self) -> usize {
        PERMUTATION_WIDTH
    }
}

impl<AB: AirBuilderWithPublicValues> Air<AB> for LineaPermutationAIR {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let pis = builder.public_values();
        let challenge = pis[0].into();

        // check[0] == (a[0] + ch) * inv[0]
        builder.when_first_row().assert_eq(
            local[PERMUTATION_COLUMN_CHECK_INDEX].into(),
            (local[PERMUTATION_COLUMN_A_INDEX] + challenge.clone())
                * local[PERMUTATION_COLUMN_INV_INDEX].into(),
        );

        // (b[i] + ch) * inv[i] == 1
        builder.when_transition().assert_eq(
            (local[PERMUTATION_COLUMN_B_INDEX] + challenge.clone())
                * local[PERMUTATION_COLUMN_INV_INDEX],
            AB::F::ONE,
        );

        // check[i+1] == inv[i+1] * (a[i+1] + ch) * check[i]
        builder.when_transition().assert_eq(
            next[PERMUTATION_COLUMN_CHECK_INDEX].into(),
            (next[PERMUTATION_COLUMN_A_INDEX] + challenge.clone())
                * next[PERMUTATION_COLUMN_INV_INDEX]
                * local[PERMUTATION_COLUMN_CHECK_INDEX],
        );

        // check[i] == 1
        builder
            .when_last_row()
            .assert_eq(local[2].into(), AB::F::ONE);
    }
}
