mod air;
use corset::{cgo, import};

fn main() {
    let mut corset = cgo::corset_from_file("zkevm.bin").unwrap();
    import::parse_binary_trace("traces/trace1.lt", &mut corset, true).unwrap();

    corset.columns.registers.iter().for_each(|r| {
        if let Some(backing) = r.backing() {
            println!("{:?}", backing);
        } else {
            println!("None");
        }
    })
}
