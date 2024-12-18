use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::Field;

pub struct LineaAir {}

impl<F: Field> BaseAir<F> for LineaAir {
    fn width(&self) -> usize {
        0
    }
}

impl<AB: AirBuilder> Air<AB> for LineaAir {
    fn eval(&self, builder: &mut AB) {}
}
