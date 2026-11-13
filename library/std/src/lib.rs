// Lust `std`: the OS layer (docs/ARCH.md section 3.1).
//
// Skeleton from serial refactor S5. A user crate's `std` dependency is
// implicit and rooted here. Module files land beside this one
// (process.rs is first); until W-RESOLVE builds the module tree, the
// std loader reads each shipped module file by path (src/stdload.rs),
// so this file declares no `mod` items yet.
