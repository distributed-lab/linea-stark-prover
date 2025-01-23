mod air_lookup;
mod air_permutation;

use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, FieldAlgebra, TwoAdicField};
use p3_matrix::Matrix;

#[derive(Clone)]
pub struct AirPermutationConfig {
    // Unimplemented for now
}

#[derive(Clone)]
pub struct AirLookupConfig {
    /// 0...a-1 it is the A columns
    pub a_shift: usize,
    /// a...2a-1 it is the A\[i]+challenge inverse
    pub a_inv_shift: usize,
    ///
    pub a_width: usize,
    /// 2a...2a+b-1 it is the B columns
    pub b_shift: usize,
    /// 2a+b...2a+2b-1 it is the B\[i]+challenge inverse
    pub b_inv_shift: usize,
    ///
    pub b_width: usize,
    /// 2a+2b ...2a+3b-1 it is the occurrences of B\[i] in A
    pub occurrences_column_shift: usize,
    /// 2a+3b it is a check lookup constrain column
    pub check_column_shift: usize,
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
            width: a_width * 2 + 3 * b_width + 1,
        }
    }
}

#[derive(Clone)]
pub enum AirConfig {
    Lookup(AirLookupConfig),
    Permutation(AirPermutationConfig),
}

impl AirConfig {
    pub fn width(&self) -> usize {
        match self {
            AirConfig::Lookup(l) => l.width,
            AirConfig::Permutation(_) => {
                unimplemented!("permutations are not currently implemented")
            }
        }
    }
}

#[derive(Clone)]
pub struct LineaAIR<F: Field> {
    pub configs: Vec<AirConfig>,
    pub challenge: F,

    matrix_width: usize,
}

impl<F: Field> LineaAIR<F> {
    pub fn new(configs: Vec<AirConfig>, challenge: F) -> Self {
        Self {
            matrix_width: configs.iter().map(|c| c.width()).sum(),

            configs,
            challenge,
        }
    }
}

impl<F: Field> BaseAir<F> for LineaAIR<F> {
    fn width(&self) -> usize {
        self.matrix_width
    }
}

impl<AB: AirBuilderWithPublicValues> Air<AB> for LineaAIR<AB::F> {
    fn eval(&self, builder: &mut AB) {
        let mut offset = 0;

        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let challenge = AB::F::from_f(self.challenge.clone());

        self.configs.iter().for_each(|c| {
            match c {
                AirConfig::Lookup(l) => {
                    for i in 0..l.a_width {
                        // 1 == (a[i] + ch) * inv_a[i]
                        builder.assert_eq(
                            AB::F::ONE,
                            (local[l.a_shift + i + offset] + challenge.clone())
                                * local[l.a_inv_shift + i + offset],
                        );
                    }

                    for i in 0..l.b_width {
                        // 1 == (b[i] + ch) * inv_b[i]
                        builder.assert_eq(
                            AB::F::ONE,
                            (local[l.b_shift + i + offset] + challenge.clone())
                                * local[l.b_inv_shift + i + offset],
                        );
                    }

                    let mut local_a_total = AB::Expr::from(AB::F::ZERO);
                    for i in 0..l.a_width {
                        local_a_total = local_a_total + local[l.a_inv_shift + i + offset];
                    }

                    let mut local_b_total = AB::Expr::from(AB::F::ZERO);
                    for i in 0..l.b_width {
                        local_b_total = local_b_total
                            + (local[l.occurrences_column_shift + i + offset]
                                * local[l.b_inv_shift + i + offset]);
                    }

                    // check[0] == 1/(a[0] + ch) - s[0]/(b[0] + ch)
                    builder.when_first_row().assert_eq(
                        local[l.check_column_shift + offset],
                        local_a_total - local_b_total,
                    );

                    let mut next_a_total = AB::Expr::from(AB::F::ZERO);
                    for i in 0..l.a_width {
                        next_a_total = next_a_total + next[l.a_inv_shift + i + offset];
                    }

                    let mut next_b_total = AB::Expr::from(AB::F::ZERO);
                    for i in 0..l.b_width {
                        next_b_total +=
                            (next[l.occurrences_column_shift + i + offset]
                                * next[l.b_inv_shift + i + offset]);
                    }

                    // check[i+1] ==  1/(a[i+1] + ch) - s[i+1]/(b[i+1] + ch)  + check[i]
                    builder.when_transition().assert_eq(
                        next[l.check_column_shift + offset],
                        next_a_total - next_b_total + local[l.check_column_shift + offset],
                    );

                    // check[i] == 1
                    builder
                        .when_last_row()
                        .assert_eq(local[l.check_column_shift + offset], AB::F::ZERO);

                    offset += l.width;
                }
                _ => unimplemented!("for now only permutation costraint is implemented"),
            }
        });
    }
}
