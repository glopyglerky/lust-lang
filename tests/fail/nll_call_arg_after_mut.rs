// expect: L0303
// rustc E0503: an earlier argument holds `&mut x` until the call runs.
fn f(b: &mut i32, a: i32) -> i32 {
    a + *b
}

fn main() {
    let mut x = 1;
    let m = &mut x;
    let y = f(m, x);
    println!("{}", y);
}
