use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, Field, PackedValue};
use p3_matrix::Matrix;
use p3_mersenne_31::Mersenne31;
use std::ops::{Add, Div};
use std::ops::Mul;
use p3_field::extension::BinomialExtensionField;
use p3_uni_stark::SymbolicVariable;

pub struct LineaAir {
    // TODO: change to BLS12-377
    pub challenge: Mersenne31,
}

impl<F: Field> BaseAir<F> for LineaAir {
    fn width(&self) -> usize {
        4
    }
}

impl<AB: AirBuilder<F = Mersenne31>> Air<AB> for LineaAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        // Column indexes: 0 - first, 1 - second, 2 - s, 3 - inverse.

        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let challenge = <AB as AirBuilder>::F::from(self.challenge);

        builder.when_first_row().assert_eq(
            local[2].into(),
            local[0].add(challenge).mul(local[3])
        );

        builder.when_transition().assert_eq(
            local[1].add(challenge).mul(local[3]),
            AB::F::one()
        );
        builder.when_transition().assert_eq(
            next[2].add(<AB as AirBuilder>::F::zero()),
            next[0].add(challenge).mul(next[3]).mul(local[2]),
        );

        builder.when_last_row().assert_eq(local[2].into(), AB::F::one());
    }
}
