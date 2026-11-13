// expect: L0303
// rustc E0503: the call reads `x` while a later argument still holds `&mut x`.
fn f(a: i32, b: &mut i32) -> i32 {
    a + *b
}

fn main() {
    let mut x = 1;
    let m = &mut x;
    let y = f(x, m);
    println!("{}", y);
}
