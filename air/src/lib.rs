pub mod air_lookup;
pub mod air_permutation;

use crate::air_permutation::AirPermutationConfig;
use air_lookup::AirLookupConfig;
use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Debug)]
pub enum AirConfig {
    Lookup(AirLookupConfig),
    Permutation(AirPermutationConfig),
}

impl AirConfig {
    pub fn width(&self) -> usize {
        match self {
            AirConfig::Lookup(l) => l.width(),
            AirConfig::Permutation(p) => p.width(),
        }
    }
}

#[derive(Clone)]
pub struct LineaAIR {
    configs: Vec<AirConfig>,
    width: usize,
}

impl LineaAIR {
    pub fn new(configs: Vec<AirConfig>) -> Self {
        Self {
            width: configs.iter().map(|c| c.width()).sum(),
            configs,
        }
    }
}

impl<F: Field> BaseAir<F> for LineaAIR {
    fn width(&self) -> usize {
        self.width
    }
}

impl<AB: AirBuilderWithPublicValues> Air<AB> for LineaAIR {
    fn eval(&self, builder: &mut AB) {
        self.configs.iter().for_each(|c| match c {
            AirConfig::Lookup(l) => self.eval_lookup(builder, l),
            AirConfig::Permutation(p) => self.eval_permutation(builder, p),
        });
    }
}

impl LineaAIR {
    fn eval_lookup<AB: AirBuilderWithPublicValues>(&self, builder: &mut AB, l: &AirLookupConfig) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let alpha = builder.public_values()[0].into();
        let delta = builder.public_values()[1].into();

        let mut a_local_comb = AB::Expr::from(AB::F::ZERO);
        for i in &l.a_columns_ids {
            a_local_comb = a_local_comb * alpha.clone() + local[*i]
        }

        let a_local_challenge = a_local_comb + delta.clone();

        // Check inverse calculated correctly
        builder.assert_eq(a_local_challenge * local[l.a_inverses_id], AB::F::ONE);

        let mut local_check = local[l.a_filter_id] * local[l.a_inverses_id];
        let mut next_check = next[l.a_filter_id] * next[l.a_inverses_id];

        for (b_table_ind, b_columns_ids) in l.b_columns_ids.iter().enumerate() {
            let mut b_local_comb = AB::Expr::from(AB::F::ZERO);
            for i in b_columns_ids {
                b_local_comb = b_local_comb * alpha.clone() + local[*i]
            }

            let b_local_challenge = b_local_comb + delta.clone();
            builder.assert_eq(
                b_local_challenge * local[l.b_inverses_id[b_table_ind]],
                AB::F::ONE,
            );

            local_check -= local[l.b_filter_id[b_table_ind]]
                * local[l.occurrences_id[b_table_ind]]
                * local[l.b_inverses_id[b_table_ind]];

            next_check -= next[l.b_filter_id[b_table_ind]]
                * next[l.occurrences_id[b_table_ind]]
                * next[l.b_inverses_id[b_table_ind]]
        }

        // Check first row calculated correctly
        builder
            .when_first_row()
            .assert_eq(local[l.check_id], local_check);

        // Check each row transition
        builder
            .when_transition()
            .assert_eq(next[l.check_id] - local[l.check_id], next_check);

        // Check total sum is zero
        builder
            .when_last_row()
            .assert_eq(local[l.check_id], AB::F::ZERO);
    }

    fn eval_permutation<AB: AirBuilderWithPublicValues>(
        &self,
        builder: &mut AB,
        p: &AirPermutationConfig,
    ) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let alpha = builder.public_values()[0].into();
        let delta = builder.public_values()[1].into();

        let mut a_local_comb = AB::Expr::from(AB::F::ZERO);
        for i in &p.a_columns_ids {
            a_local_comb = a_local_comb * alpha.clone() + local[*i]
        }

        let mut b_local_comb = AB::Expr::from(AB::F::ZERO);
        for i in &p.b_columns_ids {
            b_local_comb = b_local_comb * alpha.clone() + local[*i]
        }

        let a_local_challenge = a_local_comb + delta.clone();
        let b_local_challenge = b_local_comb + delta.clone();

        // Check inverse calculated correctly
        builder.assert_eq(b_local_challenge * local[p.b_inverse_id], AB::F::ONE);

        // Check first row calculated correctly
        builder
            .when_first_row()
            .assert_eq(local[p.check_id], a_local_challenge * local[p.b_inverse_id]);

        let mut a_next_comb = AB::Expr::from(AB::F::ZERO);
        for i in &p.a_columns_ids {
            a_next_comb = a_next_comb * alpha.clone() + next[*i]
        }

        let a_next_challenge = a_next_comb + delta.clone();

        // Check each row transition
        builder.when_transition().assert_eq(
            next[p.check_id],
            local[p.check_id] * a_next_challenge * next[p.b_inverse_id],
        );

        // Check total prod is one
        builder
            .when_last_row()
            .assert_eq(local[p.check_id], AB::F::ONE);
    }
}
