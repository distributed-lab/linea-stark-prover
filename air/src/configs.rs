use p3_field::Field;

#[derive(Clone, Debug)]
pub struct AirLookupConfig {
    pub a_columns_ids: Vec<Vec<usize>>,
    pub b_columns_ids: Vec<Vec<usize>>,
    pub a_filter_id: Option<Vec<usize>>,
    pub b_filter_id: Option<Vec<usize>>,
    pub a_inverses_id: Vec<usize>,
    pub b_inverses_id: Vec<usize>,
    pub occurrences_id: Vec<usize>,
    pub check_id: usize,
}

#[derive(Clone, Debug)]
pub struct AirPermutationConfig {
    pub a_columns_ids: Vec<usize>,
    pub b_columns_ids: Vec<usize>,
    pub b_inverse_id: usize,
    pub check_id: usize,
}

#[derive(Clone, Debug)]
pub enum AirOperatorType {
    Constant,
    LinearCombination,
    PolyEval,
    Product,
    Variable,
    Unknown
}

impl From<u8> for AirOperatorType {
    fn from(value: u8) -> Self {
        match value {
            0 => AirOperatorType::Constant,
            1 => AirOperatorType::LinearCombination,
            2 => AirOperatorType::PolyEval,
            3 => AirOperatorType::Product,
            4 => AirOperatorType::Variable,
            _ => AirOperatorType::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AirOperator<F: Field> {
    pub _type: AirOperatorType,
    /// Used for [AirOperatorType::Constant] operator.
    pub value: F,
    /// Used for [AirOperatorType::LinearCombination] and [AirOperatorType::Product] operators.
    pub coeffs: Option<Vec<u32>>,
    /// Used for [AirOperatorType::Variable] operator.
    pub id: i32,
}

#[derive(Clone, Debug)]
pub struct AirNode<F: Field> {
    pub children: Vec<u64>,
    pub operator: AirOperator<F>
}

#[derive(Clone, Debug)]
pub struct AirGlobalConfig<F: Field> {
    pub nodes: Vec<Vec<AirNode<F>>>,
    pub input_columns_ids: Vec<usize>,
    pub skip_column_id: usize,
}