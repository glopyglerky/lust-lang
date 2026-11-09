// expect: L0612
// rustc E0277: std's `ExitCode` is `PartialEq` but not `Eq`.
fn need<X: Eq>(_: &X) {}
fn main() {
    need(&std::process::ExitCode::from(3));
}
