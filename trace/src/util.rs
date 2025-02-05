use p3_bls12_377_fr::Scalar;

pub(crate) fn decode_be32(buf: [u8; 32]) -> Scalar {
    let mut value_le = buf.to_vec();
    value_le.reverse();
    Scalar::decode(value_le.as_slice()).unwrap()
}
