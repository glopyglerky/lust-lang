// expect: L0306
// rustc E0597: a `'static` slice of `&x` needs `x` to live for `'static`.
fn main() {
    let x = 5;
    let v = vec![&x];
    let s: &'static mut [&i32] = v.leak();
    println!("{:?}", s);
}
