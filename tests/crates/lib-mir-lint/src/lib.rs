// A deny-by-default MIR lint in a lib target: `cargo build --lib` rejects it,
// `cargo check --lib` (metadata only) accepts it. lust rejects it (L0541).
pub fn f() -> u32 {
    let a = [1, 2, 3];
    a[5]
}
