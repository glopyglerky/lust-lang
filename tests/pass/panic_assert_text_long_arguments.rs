// Argument, tuple, array and repeat lists break inconsistently.
// panic: assertion failed:
fn equal(a: &[i32], b: (i32, i32), c: i32) -> bool {
    a[0] == b.0 && b.1 == c
}
fn main() {
    let first_long_argument_name = 1;
    let second_long_argument_name = 2;
    let third_long_argument_name = 3;
    assert!(equal(&[first_long_argument_name; 3], (second_long_argument_name, third_long_argument_name), first_long_argument_name));
}
