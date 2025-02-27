pub mod configs;

use crate::configs::{
    AirGlobalConfig, AirLookupConfig, AirOperator, AirOperatorType, AirPermutationConfig,
};
use eyre::{bail, eyre};
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;

#[derive(Clone, Debug)]
pub enum AirConfig<F: Field> {
    /// Lookup with filters
    Lookup(AirLookupConfig),
    Permutation(AirPermutationConfig),
    Global(AirGlobalConfig<F>),
}

pub trait LineaConfigAIR<AB: AirBuilder> {
    fn eval_lookup(&self, builder: &mut AB, l: &AirLookupConfig);
    fn eval_permutation(&self, builder: &mut AB, p: &AirPermutationConfig);
    fn eval_global(&self, builder: &mut AB, g: &AirGlobalConfig<AB::F>);
    fn evaluate_linear_combination(
        &self,
        inputs: Vec<AB::Expr>,
        coeffs: Option<Vec<i32>>,
    ) -> eyre::Result<AB::Expr>;
    fn evaluate_poly_eval(&self, inputs: Vec<AB::Expr>) -> eyre::Result<AB::Expr>;
    fn evaluate_product(
        &self,
        inputs: Vec<AB::Expr>,
        coeffs: Option<Vec<i32>>,
    ) -> eyre::Result<AB::Expr>;
    fn evaluate(
        &self,
        inputs: Vec<AB::Expr>,
        operator: AirOperator<AB::F>,
    ) -> eyre::Result<AB::Expr>;
}

#[derive(Clone)]
pub struct LineaAIR<F: Field> {
    configs: Vec<AirConfig<F>>,
    width: usize,
    challenges: Vec<F>,
}

impl<F: Field> LineaAIR<F> {
    pub fn new(configs: Vec<AirConfig<F>>, width: usize, challenges: Vec<F>) -> Self {
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
            AirConfig::Global(g) => self.eval_global(builder, g),
        });
    }
}

impl<AB: AirBuilder> LineaConfigAIR<AB> for LineaAIR<AB::F> {
    fn eval_lookup(&self, builder: &mut AB, l: &AirLookupConfig) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let (alpha, delta) = (self.challenges[0], self.challenges[1]);

        let mut local_check = AB::Expr::from(AB::F::ZERO);
        let mut next_check = AB::Expr::from(AB::F::ZERO);

        for (a_table_ind, a_columns_ids) in l.a_columns_ids.iter().enumerate() {
            let mut a_local_comb = AB::Expr::from(AB::F::ZERO);
            for i in a_columns_ids {
                a_local_comb = a_local_comb * alpha + local[*i]
            }

            // Check inverse calculated correctly
            let a_local_challenge = a_local_comb + delta;
            builder.assert_eq(
                a_local_challenge * local[l.a_inverses_id[a_table_ind]],
                AB::F::ONE,
            );

            let mut local_add = local[l.a_inverses_id[a_table_ind]].into();
            let mut next_add = next[l.a_inverses_id[a_table_ind]].into();

            if let Some(a_filter_id) = l.a_filter_id.clone() {
                local_add *= local[a_filter_id[a_table_ind]].into();
                next_add *= next[a_filter_id[a_table_ind]].into();
            }

            local_check += local_add;
            next_check += next_add;
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

        let (alpha, delta) = (self.challenges[0], self.challenges[1]);

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

    fn eval_global(&self, builder: &mut AB, g: &AirGlobalConfig<AB::F>) {
        let main = builder.main();

        let local = main.row_slice(0);

        let mut intermediate_result = Vec::new();
        for i in 0..g.nodes.len() {
            let mut vec = Vec::new();
            vec.resize(g.nodes[i].len(), AB::Expr::ZERO);
            intermediate_result.push(vec);
        }

        // Store the initial values in the level entries of the vector.
        let mut input_cursor_id = 0;
        for (i, node) in g.nodes[0].iter().enumerate() {
            match &node.operator._type {
                AirOperatorType::Constant => {
                    intermediate_result[0][i] = AB::Expr::ZERO + node.operator.value
                }
                AirOperatorType::Variable => {
                    intermediate_result[0][i] =
                        AB::Expr::ZERO + local[g.input_columns_ids[input_cursor_id]];
                    input_cursor_id += 1;
                }
                _ => panic!("This operator type can not be used on zero level"),
            }
        }

        // Computes the levels one by one
        for level in 1..g.nodes.len() {
            for (pos, node) in g.nodes[level].iter().enumerate() {
                let mut node_inputs = Vec::new();
                node_inputs.resize(node.children.len(), AB::Expr::ZERO);

                for (i, child_id) in node.children.iter().enumerate() {
                    let l = get_level(*child_id as usize);
                    let p = get_pos_level(*child_id as usize);

                    node_inputs[i] = intermediate_result[l][p].clone();
                }

                let res: AB::Expr =
                    <LineaAIR<<AB as AirBuilder>::F> as LineaConfigAIR<AB>>::evaluate(
                        self,
                        node_inputs,
                        node.operator.clone(),
                    )
                    .unwrap();
                intermediate_result[level][pos] = res;
            }
        }

        assert_eq!(intermediate_result.last().unwrap().len(), 1);

        builder.assert_eq(
            intermediate_result[g.nodes.len() - 1][0].clone() * local[g.skip_column_id],
            AB::Expr::ZERO,
        );
    }

    fn evaluate(
        &self,
        inputs: Vec<AB::Expr>,
        operator: AirOperator<AB::F>,
    ) -> eyre::Result<AB::Expr> {
        match operator._type {
            AirOperatorType::LinearCombination => <LineaAIR<<AB as AirBuilder>::F> as LineaConfigAIR<AB>>::evaluate_linear_combination(self, inputs, operator.coeffs),
            AirOperatorType::PolyEval => <LineaAIR<<AB as AirBuilder>::F> as LineaConfigAIR<AB>>::evaluate_poly_eval(self, inputs),
            AirOperatorType::Product => <LineaAIR<<AB as AirBuilder>::F> as LineaConfigAIR<AB>>::evaluate_product(self, inputs, operator.coeffs),
            _ => bail!("evaluate should be never called on Constant or Variable or the provided operator is unknown"),
        }
    }

    fn evaluate_linear_combination(
        &self,
        inputs: Vec<AB::Expr>,
        coeffs_opt: Option<Vec<i32>>,
    ) -> eyre::Result<AB::Expr> {
        let coeffs = coeffs_opt.clone().ok_or(eyre!(
            "coefficients for linear combination should not be None"
        ))?;
        if inputs.len() != coeffs.len() {
            bail!("number of inputs should be equal to the number of coefficients: {} inputs but {} coefficients", inputs.len(), coeffs.len())
        }

        let mut res = AB::Expr::ZERO;
        for (i, input) in inputs.iter().enumerate() {
            let mut coeff = AB::Expr::from_canonical_u32(coeffs[i].unsigned_abs());
            if coeffs[i] < 0 {
                coeff = -coeff;
            }

            res += input.clone() * coeff;
        }

        Ok(res)
    }

    fn evaluate_poly_eval(&self, inputs: Vec<AB::Expr>) -> eyre::Result<AB::Expr> {
        let x = inputs
            .first()
            .ok_or(eyre::eyre!(
                "can't get the first element since input vector is empty"
            ))?
            .clone();
        let mut res = inputs
            .last()
            .ok_or(eyre::eyre!(
                "can't get the last element since input vector is empty"
            ))?
            .clone();

        let mut i = inputs.len() - 2;
        while i >= 1 {
            res *= x.clone();
            res += inputs[i].clone();
            i -= 1;
        }

        Ok(res)
    }

    fn evaluate_product(
        &self,
        inputs: Vec<AB::Expr>,
        coeffs_opt: Option<Vec<i32>>,
    ) -> eyre::Result<AB::Expr> {
        let coeffs = coeffs_opt.clone().ok_or(eyre!(
            "coefficients for linear combination should not be None"
        ))?;
        if inputs.len() != coeffs.len() {
            bail!("number of inputs should be equal to the number of coefficients: {} inputs but {} coefficients", inputs.len(), coeffs.len())
        }

        let mut res = AB::Expr::ONE;
        for (i, input) in inputs.iter().enumerate() {
            res *= input.exp_u64(coeffs[i] as u64);
        }

        Ok(res)
    }
}

fn get_level(num: usize) -> usize {
    num >> 32
}

fn get_pos_level(num: usize) -> usize {
    num & ((1 << 32) - 1)
}
