// expect: L0612
// rustc E0277: `&mut dyn FnMut` is not `Fn` (core forwards `Fn` only
// through `&F` with `F: Fn`).
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}
fn main() {
    let mut n = 0;
    let m: &mut dyn FnMut(i32) -> i32 = &mut |x| {
        n += 1;
        x + n
    };
    println!("{}", apply(m, 1));
}
