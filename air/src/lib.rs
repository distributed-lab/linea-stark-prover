pub mod air_lookup;
pub mod air_permutation;

use crate::air_permutation::AirPermutationConfig;
use air_lookup::AirLookupConfig;
use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;
// use air_permutation::AirPermutationConfig;

#[derive(Clone, Debug)]
pub enum AirConfig {
    Lookup(AirLookupConfig),
    Permutation(AirPermutationConfig),
}

impl AirConfig {
    pub fn width(&self) -> usize {
        match self {
            AirConfig::Lookup(l) => l.width,
            AirConfig::Permutation(p) => p.get_width(),
        }
    }
}

#[derive(Clone)]
pub struct LineaAIR {
    pub configs: Vec<AirConfig>,
    matrix_width: usize,
}

impl LineaAIR {
    pub fn new(configs: Vec<AirConfig>) -> Self {
        Self {
            matrix_width: configs.iter().map(|c| c.width()).sum(),

            configs,
        }
    }
}

impl<F: Field> BaseAir<F> for LineaAIR {
    fn width(&self) -> usize {
        println!("{}", self.matrix_width);
        self.matrix_width
        // 14
    }
}

impl<AB: AirBuilderWithPublicValues> Air<AB> for LineaAIR {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let challenge = builder.public_values()[0].into();

        let mut offset = 0;

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
                        local_b_total += local[l.occurrences_column_shift + i + offset]
                            * local[l.b_inv_shift + i + offset];
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
                        next_b_total += next[l.occurrences_column_shift + i + offset]
                            * next[l.b_inv_shift + i + offset];
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
                AirConfig::Permutation(p) => {
                    let mut local_a_total = AB::Expr::from(AB::F::ONE);
                    for i in 0..p.width {
                        local_a_total *= local[i + offset] + challenge.clone();
                    }

                    let mut local_b_total = AB::Expr::from(AB::F::ONE);
                    for i in p.width..p.width * 2 {
                        // println!("i val: {}", i);
                        local_b_total *= local[i + offset] + challenge.clone();
                    }

                    // check[0] == (a[0] + ch) * inv[0]
                    builder.when_first_row().assert_eq(
                        local[p.check_column_index + offset],
                        local_a_total * local[p.inv_column_index + offset],
                    );

                    // (b[i] + ch) * inv[i] == 1
                    builder.assert_eq(
                        local_b_total * local[p.inv_column_index + offset],
                        AB::F::ONE,
                    );

                    let mut next_a_total = AB::Expr::from(AB::F::ONE);
                    for i in 0..p.width {
                        next_a_total *= next[i + offset] + challenge.clone();
                    }

                    let mut next_b_total = AB::Expr::from(AB::F::ONE);
                    for i in p.width..p.width * 2 {
                        next_b_total *= next[i + offset] + challenge.clone();
                    }

                    // check[i+1] == inv[i+1] * (a[i+1] + ch) * check[i]
                    builder.when_transition().assert_eq(
                        next[p.check_column_index + offset],
                        next[p.inv_column_index + offset]
                            * next_a_total
                            * local[p.check_column_index + offset],
                    );

                    // check[i] == 1
                    builder
                        .when_last_row()
                        .assert_eq(local[p.check_column_index + offset], AB::F::ONE);

                    offset += p.get_width();
                }
            }
        });
    }
}
