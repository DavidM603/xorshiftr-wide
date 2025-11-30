# xorshiftr-wide

A high-throughput PRNG, designed to autovectorize well and fill buffers. Currently, filling `&mut [u64]` is supported, and it is up to the user to adapt these random bits to their needs. Compiling with a target-cpu set in your RUSTFLAGS is recommended.

To use this crate, you will need a source of randomness for seeding; some suggested options are [getrandom](https://crates.io/crates/getrandom) and [rand](https://crates.io/crates/rand)'s `rng()`.

The design is largely based on the paper [A random number generator for lightweight authentication protocols: xorshiftR+](https://www.researchgate.net/publication/362606255_A_random_number_generator_for_lightweight_authentication_protocols_xorshiftR). Their default shift constants and ordering passed BigCrush. The modified shift constants and ordering used in xorshiftr-wide last longer in PractRand, and have passed 32TB with 16 lanes. Testing is underway in search of further improvements.