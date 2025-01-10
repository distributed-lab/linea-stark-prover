mod air;
use corset::cgo::Trace;
use corset::{cgo, import};

fn main() {
    let mut corset = cgo::corset_from_file("zkevm.bin").unwrap();
    import::parse_binary_trace(
        "traces/4181195-4181272.conflated.v0.8.0-rc3.lt",
        &mut corset,
        true,
    )
    .unwrap();

    let trace = Trace::from_constraints(&corset);
    println!("{:?}", trace.ids)
}
