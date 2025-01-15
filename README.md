# Plonky3 Linea prover

## How to run?

```bash
cargo run --release --feature parallel
```

## Benchmarks

Proving of the permutation constrain over 3x3 columns of 524288 elements takes ~330s to prove and <1s to verify. 

Benchmarking has been done in the following environment:
```log
Architecture:             x86_64
  CPU op-mode(s):         32-bit, 64-bit
  Address sizes:          40 bits physical, 48 bits virtual
  Byte Order:             Little Endian
CPU(s):                   24
  On-line CPU(s) list:    0-2,4-10,12-17
  Off-line CPU(s) list:   3,11,18-23
```

The memory usage during all prove was less than 3Gb RAM.