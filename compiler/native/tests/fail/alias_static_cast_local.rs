// expect: L0306
// rustc E0597: `&n as R` needs `n` to live for `'static`: the cast target keeps the alias's
// `'static` (a cast annotation).
type R = &'static u32;
fn main() {
    let n = 5u32;
    let x = &n as R;
    println!("{}", x);
}
