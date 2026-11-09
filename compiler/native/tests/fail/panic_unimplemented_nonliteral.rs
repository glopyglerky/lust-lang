// expect: L0806
// rustc: format argument must be a string literal.
fn main() {
    let m = "mode";
    unimplemented!(m);
}
