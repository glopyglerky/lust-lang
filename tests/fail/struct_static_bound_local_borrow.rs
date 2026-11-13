// expect: L0306
// rustc E0597: `S<&'a i32>` is well formed only if `'a: 'static` (the
// struct's `T: 'static` bound); `x` is a local.
struct S<T: 'static> { t: T }
fn main() {
    let x = 5;
    let s = S { t: &x };
    println!("{}", s.t);
}
