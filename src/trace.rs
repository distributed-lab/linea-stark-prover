use p3_bls12_377_fr::{Bls12_377Fr, FF_Bls12_377Fr};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use std::collections::HashMap;
use std::fs;

pub fn generate_permutation_trace(
    a: Vec<Vec<Bls12_377Fr>>,
    b: Vec<Vec<Bls12_377Fr>>,
    challenge: Bls12_377Fr,
    sz: usize,
) -> RowMajorMatrix<Bls12_377Fr> {
    let mut res: Vec<Bls12_377Fr> = Vec::new();
    let mut prev_check = Bls12_377Fr::ONE;

    for i in 0..sz {
        let mut a_total = Bls12_377Fr::ONE;
        let mut b_total = Bls12_377Fr::ONE;

        for j in 0..a.len() {
            res.push(a[j][i].clone());
            a_total = a_total * (a[j][i] + challenge);
        }

        for j in 0..b.len() {
            res.push(b[j][i].clone());
            b_total = b_total * (b[j][i] + challenge);
        }

        let b_total_inverse = b_total.inverse();
        prev_check = prev_check * a_total * b_total_inverse;
        res.push(prev_check);
        res.push(b_total_inverse);
    }

    assert!(
        res.get(res.len() - 2).unwrap().is_one(),
        "failed to check constrain: check column should be 1 on the last row"
    );
    assert_eq!(a.len(), b.len(), "a 'from' number of columns should be equal to the 'to'");
    RowMajorMatrix::new(res, a.len() + b.len() + 2)
}

pub fn read_trace(path: &str) -> HashMap<String, Vec<Bls12_377Fr>> {
    let file_content = fs::read_to_string(path).unwrap();
    let raw_trace: HashMap<String, Vec<u128>> = serde_json::from_str(&file_content).unwrap();

    raw_trace
        .into_iter()
        .map(|(name, column)| {
            let parsed_column: Vec<Bls12_377Fr> = column
                .into_iter()
                .map(|num| Bls12_377Fr::new(FF_Bls12_377Fr::from(num)))
                .collect();

            (name, parsed_column)
        })
        .collect()
}
