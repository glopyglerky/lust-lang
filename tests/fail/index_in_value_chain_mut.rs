// expect: L0308
// rustc E0596: `iter_mut` needs `&mut [u8]`, and `s[1..]` is behind `&`.
fn f(s: &[u8]) -> Option<u8> {
    s[1..].iter_mut().next().map(|x| *x)
}
fn main() {
    println!("{:?}", f(&[1, 2]));
}
