pub mod configs;

use crate::configs::{AirLookupConfig, AirPermutationConfig};
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;

#[derive(Clone, Debug)]
pub enum AirConfig {
    /// Lookup with filters
    Lookup(AirLookupConfig),
    Permutation(AirPermutationConfig),
}

pub trait LineaConfigAIR<AB: AirBuilder> {
    fn eval_lookup(&self, builder: &mut AB, l: &AirLookupConfig);
    fn eval_permutation(&self, builder: &mut AB, p: &AirPermutationConfig);
}

#[derive(Clone)]
pub struct LineaAIR<F> {
    configs: Vec<AirConfig>,
    width: usize,
    challenges: Vec<F>,
}

impl<F: Field> LineaAIR<F> {
    pub fn new(configs: Vec<AirConfig>, width: usize, challenges: Vec<F>) -> Self {
        Self {
            configs,
            width,
            challenges,
        }
    }
}

impl<F: Field> BaseAir<F> for LineaAIR<F> {
    fn width(&self) -> usize {
        self.width
    }
}

impl<AB: AirBuilder> Air<AB> for LineaAIR<AB::F> {
    fn eval(&self, builder: &mut AB) {
        self.configs.iter().for_each(|c| match c {
            AirConfig::Lookup(l) => self.eval_lookup(builder, l),
            AirConfig::Permutation(p) => self.eval_permutation(builder, p),
        });
    }
}

impl<AB: AirBuilder> LineaConfigAIR<AB> for LineaAIR<AB::F>  {
    fn eval_lookup(&self, builder: &mut AB, l: &AirLookupConfig) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let (alpha,delta) = (self.challenges[0], self.challenges[1]);

        let mut a_local_comb = AB::Expr::from(AB::F::ZERO);
        for i in &l.a_columns_ids {
            a_local_comb = a_local_comb * alpha + local[*i]
        }

        let a_local_challenge = a_local_comb + delta;

        // Check inverse calculated correctly
        builder.assert_eq(a_local_challenge * local[l.a_inverses_id], AB::F::ONE);

        let mut local_check = local[l.a_inverses_id].into();
        let mut next_check = next[l.a_inverses_id].into();

        if let Some(a_filter_id) = l.a_filter_id {
            local_check *= local[a_filter_id].into();
            next_check *= next[a_filter_id].into();
        }

        // TODO: we can check then whether it will be faster to put under option the whole for loop.

        for (b_table_ind, b_columns_ids) in l.b_columns_ids.iter().enumerate() {
            let mut b_local_comb = AB::Expr::from(AB::F::ZERO);
            for i in b_columns_ids {
                b_local_comb = b_local_comb * alpha + local[*i]
            }

            let b_local_challenge = b_local_comb + delta;
            builder.assert_eq(
                b_local_challenge * local[l.b_inverses_id[b_table_ind]],
                AB::F::ONE,
            );

            let mut b_filter_local = AB::Expr::from(AB::F::ONE);
            let mut b_filter_next = AB::Expr::from(AB::F::ONE);

            if let Some(b_filter_id) = &l.b_filter_id {
                b_filter_local = local[b_filter_id[b_table_ind]].into();
                b_filter_next = next[b_filter_id[b_table_ind]].into();
            }

            local_check -= b_filter_local
                * local[l.occurrences_id[b_table_ind]].into()
                * local[l.b_inverses_id[b_table_ind]].into();

            next_check -= b_filter_next
                * next[l.occurrences_id[b_table_ind]]
                * next[l.b_inverses_id[b_table_ind]];
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

    fn eval_permutation(&self, builder: &mut AB, p: &AirPermutationConfig) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let (alpha,delta) = (self.challenges[0], self.challenges[1]);

        let mut a_local_comb = AB::Expr::from(AB::F::ZERO);
        for i in &p.a_columns_ids {
            a_local_comb = a_local_comb * alpha + local[*i]
        }

        let mut b_local_comb = AB::Expr::from(AB::F::ZERO);
        for i in &p.b_columns_ids {
            b_local_comb = b_local_comb * alpha + local[*i]
        }

        let a_local_challenge = a_local_comb + delta;
        let b_local_challenge = b_local_comb + delta;

        // Check inverse calculated correctly
        builder.assert_eq(b_local_challenge * local[p.b_inverse_id], AB::F::ONE);

        // Check first row calculated correctly
        builder
            .when_first_row()
            .assert_eq(local[p.check_id], a_local_challenge * local[p.b_inverse_id]);

        let mut a_next_comb = AB::Expr::from(AB::F::ZERO);
        for i in &p.a_columns_ids {
            a_next_comb = a_next_comb * alpha + next[*i]
        }

        let a_next_challenge = a_next_comb + delta;

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
