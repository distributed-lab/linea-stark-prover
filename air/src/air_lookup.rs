use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_bls12_377_fr::Bls12_377Fr;
use p3_field::{Field, FieldAlgebra, TwoAdicField};
use p3_matrix::Matrix;

/// Processes the permutation for two columns
pub struct LineaLookupAIR<F: Field> {
    // 0...a-1 it is the A columns
    pub a_shift: usize,
    // a...2a-1 it is the A[i]+challenge inverse
    pub a_inv_shift: usize,
    //
    pub a_width: usize,
    // 2a...2a+b-1 it is the B columns
    pub b_shift: usize,
    // 2a+b...2a+2b-1 it is the B[i]+challenge inverse
    pub b_inv_shift: usize,
    //
    pub b_width: usize,
    // 2a+2b ...2a+3b-1 it is the occurrences of B[i] in A
    pub occurrences_column_shift: usize,
    // 2a+3b it is a check lookup constrain column
    pub check_column_shift: usize,
    pub challenge: F,
}

/// | 0.       | 1.                        | 2.       | 3.                      | 4.                        | 5.                      |
/// |`Column A`| `A[i] + challenge inverse`|`Column B`| `B[i]+challenge inverse`| `Occurrences of b[i] in a`| `Constrain check column`|

impl<F: Field> LineaLookupAIR<F> {
    pub fn new(a_width: usize, b_width: usize, challenge: F) -> Self {
        LineaLookupAIR {
            a_shift: 0,
            a_inv_shift: a_width,
            a_width,
            b_shift: a_width * 2,
            b_inv_shift: a_width * 2 + b_width,
            b_width,
            occurrences_column_shift: a_width * 2 + 2 * b_width,
            check_column_shift: a_width * 2 + 3 * b_width,
            challenge,
        }
    }
}
impl<F: Field> BaseAir<F> for LineaLookupAIR<F> {
    fn width(&self) -> usize {
        self.a_width * 2 + self.b_width * 3 + 1
    }
}

impl<AB: AirBuilderWithPublicValues> Air<AB> for LineaLookupAIR<AB::F> {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let challenge = AB::F::from_f(self.challenge.clone());

        for i in 0..self.a_width {
            // 1 == (a[i] + ch) * inv_a[i]
            builder.assert_eq(
                AB::F::ONE,
                (local[self.a_shift + i] + challenge.clone()) * local[self.a_inv_shift + i],
            );
        }

        for i in 0..self.b_width {
            // 1 == (b[i] + ch) * inv_b[i]
            builder.assert_eq(
                AB::F::ONE,
                (local[self.b_shift + i] + challenge.clone()) * local[self.b_inv_shift + i],
            );
        }

        let mut local_a_total = AB::Expr::from(AB::F::ZERO);
        for i in 0..self.a_width {
            local_a_total = local_a_total + local[self.a_inv_shift + i];
        }

        let mut local_b_total = AB::Expr::from(AB::F::ZERO);
        for i in 0..self.b_width {
            local_b_total = local_b_total
                + (local[self.occurrences_column_shift + i] * local[self.b_inv_shift + i]);
        }

        // check[0] == 1/(a[0] + ch) - s[0]/(b[0] + ch)
        builder.when_first_row().assert_eq(
            local[self.check_column_shift],
            local_a_total - local_b_total,
        );

        let mut next_a_total = AB::Expr::from(AB::F::ZERO);
        for i in 0..self.a_width {
            next_a_total = next_a_total + next[self.a_inv_shift + i];
        }

        let mut next_b_total = AB::Expr::from(AB::F::ZERO);
        for i in 0..self.b_width {
            next_b_total = next_b_total
                + (next[self.occurrences_column_shift + i] * next[self.b_inv_shift + i]);
        }

        // check[i+1] ==  1/(a[i+1] + ch) - s[i+1]/(b[i+1] + ch)  + check[i]
        builder.when_transition().assert_eq(
            next[self.check_column_shift],
            next_a_total - next_b_total + local[self.check_column_shift],
        );

        // check[i] == 1
        builder
            .when_last_row()
            .assert_eq(local[self.check_column_shift], AB::F::ZERO);
    }
}
