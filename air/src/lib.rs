pub mod air_lookup;
pub mod air_permutation;

use std::ops::{Add, Mul, Sub};
use crate::air_permutation::AirPermutationConfig;
use air_lookup::AirLookupConfig;
use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;

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
                    // f[i] = F[i][0]*1 + F[i][1]*alpha + F[i][2]*alpha^2 + …
                    let mut f = AB::Expr::from(AB::F::ZERO);
                    for i in 0..l.a_width {
                        f = f + local[l.a_shift + i + offset] * challenge.exp_u64(i as u64);
                    }

                    // t[i] = T[i][0]*1 + T[i][1]*alpha + T[i][2]*alpha^2 + …
                    let mut t = AB::Expr::from(AB::F::ZERO);
                    for i in 0..l.b_width {
                        t = t + local[l.b_shift + i + offset] * challenge.exp_u64(i as u64);
                    }

                    // f[i] + u
                    let f_u = f.clone() + challenge.clone();

                    // t[i] + u
                    let t_u = t.clone() + challenge.clone();

                    // с[i] == filter_f[i]/(f[i] + u) - filter_t[i] * s[i]/(t[i]+u)
                    builder
                        .when_first_row()
                        .assert_eq(
                            local[l.check_column_shift + offset] * f_u.clone() * t_u.clone(),
                            local[l.a_filter_column_shift + offset] * t_u -
                               local[l.b_filter_column_shift + offset] *
                                   local[l.occurrences_column_shift + offset] * f_u
                        );

                    // f[i + 1] + u
                    let f_u_next = f.add(challenge.clone());

                    // t[i + 1] + u
                    let t_u_next = t.add(challenge.clone());
                    builder.when_transition()
                        .assert_eq(
                            next[l.check_column_shift + offset] - local[l.check_column_shift + offset]
                                * f_u_next.clone() * t_u_next.clone(),
                            next[l.a_filter_column_shift + offset] * t_u_next -
                                next[l.b_filter_column_shift + offset] *
                                    next[l.occurrences_column_shift + offset] * f_u_next
                        );

                    // c[i] == 0
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
