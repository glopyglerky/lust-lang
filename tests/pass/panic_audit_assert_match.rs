// rustc prints a comma after each non-block match arm and none after a
// block arm; `>>` and `move` keep their spacing.
// panic: assertion failed: match n { 1 => { n > 5 } _ => n >> 1 == 9, }
fn main() {
    let n = std::env::args().count();
    assert!(match n { 1 => false, 2 => { false }, _ => true } || n == 1);
    assert!(v_any(n));
    assert!(n << 2 != 0 && (n >> 1) == 0);
    assert!(match n { 1 => { n > 5 } _ => n >> 1 == 9 });
}
fn v_any(n: usize) -> bool {
    let v = vec![1, 2];
    assert!(v.iter().any(move |&x| x == n));
    true
}
