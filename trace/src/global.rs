use std::cmp::max;
use air::configs::{AirGlobalConfig, AirNode, AirOperator, AirOperatorType, AirPermutationConfig};
use ark_ff::PrimeField;
use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use ciborium::Value;
use p3_air::{Air, AirBuilder};
use p3_field::{Field, FieldAlgebra};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawOperator {
    /// 0 - constant, 1 - lin, 2 - poly, 3 - prod, 4 - var
    pub Typ: u8,
    pub Value: [u8; 32],
    pub Coeffs: Option<Vec<i32>>,
    pub Id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawNode {
    pub Children: Vec<u64>,
    pub Operator: RawOperator,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawGlobalTrace {
    pub InputsIds: Vec<String>,
    pub Inputs: Vec<Vec<[u8; 32]>>,
    pub Nodes: Vec<Vec<RawNode>>,
    pub Start: usize,
    pub Stop: usize,
}


impl RawGlobalTrace {
    pub fn read_file(path: &str) -> Result<RawGlobalTrace, std::io::Error> {
        let file_content = fs::read(path)?;
        let raw_trace: RawGlobalTrace =
            ciborium::from_reader(std::io::Cursor::new(file_content)).unwrap();
        Ok(raw_trace)
    }

    pub(crate) fn resize(&mut self, size: usize) {
        for e in &mut self.Inputs {
            e.resize(size, [0u8; 32]);
        }
    }

    pub fn get_max_height(&self) -> usize {
        let mut max_height = 0_usize;
        self.Inputs.iter().for_each(|g| {
            max_height = max(max_height, g.len());
        });

        max_height
    }

    pub fn update_registry(
        &self,
        columns_registry: &mut HashMap<String, usize>,
        columns: &mut Vec<Vec<Bls12_377Fr>>,
    ) -> AirGlobalConfig<Bls12_377Fr> {
        let mut next_id = || -> usize {
            columns.push(Vec::new());
            columns.len() - 1
        };

        let mut input_columns_ids = Vec::new();
        for i in 0..self.Inputs.len() {
            let id = next_id();
            columns_registry.insert(self.InputsIds[i].clone(), id);
            input_columns_ids.push(id);
        }

        let skip_column_id = next_id();
        columns_registry.insert("skip".to_string(), skip_column_id);

        AirGlobalConfig {
            nodes: self
                .Nodes
                .iter()
                .map(|nodes| nodes.iter().map(|node| node.clone().into()).collect())
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

        for i in 0..inputs.len() {
            columns[cfg.input_columns_ids[i]] = inputs[i].clone();

            for j in 0..inputs[i].len() {
                let skip_val = if (self.Start..self.Stop).contains(&j) {
                    Bls12_377Fr::ONE
                } else {
                    Bls12_377Fr::ZERO
                };

                columns[cfg.skip_column_id].push(skip_val);
            }
        }
    }

    fn get_level(&self, num: usize) -> usize {
        num >> 32
    }

    fn get_pos_level(&self, num: usize) -> usize {
        num & ((1 << 32) - 1)
    }

    fn get_inputs(&self) -> Vec<Vec<Bls12_377Fr>> {
        self.Inputs
            .iter()
            .map(|col_inputs| {
                col_inputs
                    .iter()
                    .map(|input| Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(input)))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

}

impl Into<AirNode<Bls12_377Fr>> for RawNode {
    fn into(self) -> AirNode<Bls12_377Fr> {
        let value = Bls12_377Fr::new(FF_Bls12_377Fr::from_be_bytes_mod_order(
            &self.Operator.Value,
        ));

        AirNode {
            children: self.Children,
            operator: AirOperator {
                _type: AirOperatorType::from(self.Operator.Typ),
                value,
                coeffs: self.Operator.Coeffs.map(|coeffs| coeffs.iter().map(|coef| coef.unsigned_abs()).collect()),
                id: self.Operator.Id,
            },
        }
    }
}
