use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;

/// Processes the permutation for two columns
pub struct LineaPermutationAIR<F: Field> {
    // Tables has the same sizes
    pub width: usize,
    // width * 2 + 1
    pub inv_column_index: usize,
    // width * 2
    pub check_column_index: usize,
    pub challenge: F,
}

/// | 0.       | 1.       | 2.                     | 3.                         |
/// |`Column A`|`Column B`|`Constrain check column`|`B+challenge inverse column`|

impl<F: Field> BaseAir<F> for LineaPermutationAIR<F> {
    fn width(&self) -> usize {
        // a cols + b cols + check col + inv col
        self.width * 2 + 2
    }
}

impl<AB: AirBuilderWithPublicValues> Air<AB> for LineaPermutationAIR<AB::F> {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let challenge = AB::F::from_f(self.challenge);

        let mut local_a_total = AB::Expr::from(AB::F::ONE);
        for i in 0..self.width {
            local_a_total *= local[i] + challenge;
        }

        let mut local_b_total = AB::Expr::from(AB::F::ONE);
        for i in self.width..self.width * 2 {
            local_b_total *= local[i] + challenge;
        }

        // check[0] == (a[0] + ch) * inv[0]
        builder.when_first_row().assert_eq(
            local[self.check_column_index],
            local_a_total * local[self.inv_column_index],
        );

        // (b[i] + ch) * inv[i] == 1
        builder
            .when_transition()
            .assert_eq(local_b_total * local[self.inv_column_index], AB::F::ONE);

        let mut next_a_total = AB::Expr::from(AB::F::ONE);
        for i in 0..self.width {
            next_a_total *= next[i] + challenge;
        }

        let mut next_b_total = AB::Expr::from(AB::F::ONE);
        for i in self.width..self.width * 2 {
            next_b_total *= next[i] + challenge;
        }

        // check[i+1] == inv[i+1] * (a[i+1] + ch) * check[i]
        builder.when_transition().assert_eq(
            next[self.check_column_index],
            next[self.inv_column_index] * next_a_total * local[self.check_column_index],
        );

        // check[i] == 1
        builder
            .when_last_row()
            .assert_eq(local[self.check_column_index], AB::F::ONE);
    }
}
