use air::configs::{AirGlobalConfig, AirNode, AirOperator, AirOperatorType};
use air::LineaAIR;
use ark_ff::PrimeField;
use p3_air::{Air, BaseAir};
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::FieldAlgebra;
use p3_uni_stark::{SymbolicAirBuilder, SymbolicExpression};
use p3_util::log2_ceil_usize;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawOperator {
    /// 0 - constant, 1 - lin, 2 - poly, 3 - prod, 4 - var
    pub typ: u8,
    pub value: [u8; 32],
    pub coeffs: Option<Vec<i32>>,
    pub id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawNode {
    pub children: Vec<u64>,
    pub operator: RawOperator,
}

impl Default for RawNode {
    fn default() -> Self {
        RawNode {
            // Initialize fields here
            children: vec![],
            operator: RawOperator {
                typ: 0,
                value: [0; 32],
                coeffs: None,
                id: 0,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawGlobalTrace {
    pub inputs_ids: Vec<String>,
    pub inputs: Vec<Vec<[u8; 32]>>,
    pub nodes: Vec<Vec<RawNode>>,
    pub start: usize,
    pub stop: usize,
}

impl RawGlobalTrace {
    pub fn read_file(path: &str) -> Result<RawGlobalTrace, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawGlobalTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();
        Ok(raw_trace)
    }

    pub(crate) fn resize(&mut self, size: usize) {
        for e in &mut self.inputs {
            e.resize(size, [0u8; 32]);
        }
    }

    pub fn get_max_height(&self) -> usize {
        let mut max_height = 0_usize;
        self.inputs.iter().for_each(|g| {
            max_height = max(max_height, g.len());
        });

        max_height
    }

    pub fn update_registry(
        &mut self,
        columns_registry: &mut HashMap<String, usize>,
        columns: &mut Vec<Vec<Bls12_377Fr>>,
    ) -> AirGlobalConfig<Bls12_377Fr> {
        let mut next_id = || -> usize {
            columns.push(Vec::new());
            columns.len() - 1
        };

        // TODO check name exist, do not forget to skip for empty names
        let mut input_columns_ids = Vec::new();
        for i in 0..self.inputs.len() {
            if !self.inputs_ids[i].is_empty() && columns_registry.get(&self.inputs_ids[i]).is_some()
            {
                input_columns_ids.push(*columns_registry.get(&self.inputs_ids[i]).unwrap());
            } else {
                let id = next_id();
                columns_registry.insert(self.inputs_ids[i].clone(), id);
                input_columns_ids.push(id);
            }
        }

        let skip_column_id = next_id();
        columns_registry.insert("skip".to_string(), skip_column_id);

        AirGlobalConfig {
            nodes: self
                .nodes
                .iter_mut()
                .map(|nodes| {
                    nodes
                        .iter_mut()
                        .map(|node| std::mem::take(node).into())
                        .collect()
                })
                .collect(),
            input_columns_ids,
            skip_column_id,
        }
    }

    pub fn set_trace(
        &self,
        challenges: Vec<Bls12_377Fr>,
        columns: &mut [Vec<Bls12_377Fr>],
        cfg: &AirGlobalConfig<Bls12_377Fr>,
    ) {
        assert_eq!(
            challenges.len(),
            2,
            "Two challenges should be provided for the lookup trace"
        );

        let mut inputs = self.get_inputs();
        let sz = inputs[0].len();
        for i in 0..inputs.len() {
            columns[cfg.input_columns_ids[i]] = std::mem::take(&mut inputs[i]);
        }

        for j in 0..sz {
            let skip_val = if j >= self.start && j < self.stop {
                Bls12_377Fr::ONE
            } else {
                Bls12_377Fr::ZERO
            };

            columns[cfg.skip_column_id].push(skip_val);
        }
    }

    fn get_inputs(&self) -> Vec<Vec<Bls12_377Fr>> {
        self.inputs
            .iter()
            .map(|col_inputs| {
                col_inputs
                    .iter()
                    .map(|input| Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(input)))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn get_expression_height(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_expression_width(&self) -> usize {
        self.nodes.get(0).map_or_else(|| 0, |nodes| nodes.len())
    }

    pub fn get_min_blowup(&self, air: LineaAIR<Bls12_377Fr>, num_public: usize) -> usize {
        let mut builder = SymbolicAirBuilder::new(0, air.width(), num_public);
        air.eval(&mut builder);
        let symbolic_constraints = builder.constraints();

        let constraint_degree = symbolic_constraints
            .iter()
            .map(SymbolicExpression::degree_multiple)
            .max()
            .unwrap_or(0);

        // Increase log blowup for higher security
        let mut log = log2_ceil_usize(constraint_degree - 1);
        if log == 1 {
            log = 2
        }

        log
    }
}

impl From<RawNode> for AirNode<Bls12_377Fr> {
    fn from(val: RawNode) -> Self {
        let value = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(&val.operator.value));

        AirNode {
            children: val.children,
            operator: AirOperator {
                _type: AirOperatorType::from(val.operator.typ),
                value,
                coeffs: val.operator.coeffs,
                id: val.operator.id,
            },
        }
    }
}
