// expect: L0104
// rustc E0599: `set_mode` is a PermissionsExt method; the trait is not in scope.
fn main() {
    let mut p = std::fs::metadata("/").unwrap().permissions();
    p.set_mode(0o700);
}
