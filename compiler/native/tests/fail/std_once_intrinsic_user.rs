// expect: L0642
// rustc E0425: `once_init` is lust's std-only builtin, not a user path.
fn main() {
    let x: Option<u8> = None;
    core::intrinsics::once_init(&x, 5);
}
