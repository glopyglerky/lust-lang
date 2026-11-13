// expect: L0303
// rustc E0503: `x` is read before `*m`, while `m` still holds `&mut x`.
fn main() {
    let mut x = 1;
    let m = &mut x;
    let y = x + *m;
    println!("{}", y);
}
