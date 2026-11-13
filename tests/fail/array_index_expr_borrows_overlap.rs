// expect: L0304
// rustc E0499: index expressions are not constant indexes; arr[0] and
// arr[1] may overlap for the borrow checker.
fn main() {
    let mut arr = [1, 2, 3];
    let a = &mut arr[0];
    let b = &mut arr[1];
    *a += *b;
}
