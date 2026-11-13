// expect: L0524
// rustc: the `track_caller` attribute cannot be used on statements or
// expressions (functions only; a match arm only warns).
fn g() {}
fn main() {
    #[track_caller]
    g();
}
