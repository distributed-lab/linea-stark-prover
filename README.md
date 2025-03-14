# Plonky3 Linea prover

## How to run?

```bash
cargo run --release --features parallel
```

## Benchmarks

Proving of the permutation 67k globals, 1500 lookups and 2300 ranges (received from the default trace) 
takes about 38 minutes and 770 GB RAM at peak.