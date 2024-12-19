mod air;
use corset::cgo;

fn main() {
    let corset = cgo::corset_from_file("zkevm.bin").unwrap();
}
