// expect: L0306
// rustc E0597: the bound writes `&'a i32`, so `x` is `z`'s borrow, which
// `out` keeps past `z`.
fn apply<'a, F: FnMut(&'a i32)>(mut f: F, z: &'a i32) {
    f(z);
}
fn main() {
    let mut out: Vec<&i32> = Vec::new();
    {
        let z = 3;
        apply(|x| out.push(x), &z);
    }
    println!("{}", out.len());
}
