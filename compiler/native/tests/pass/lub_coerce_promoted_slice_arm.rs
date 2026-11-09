// W-BORROW's report shapes: a `&[]` arm or branch before a slice one
// coerces to the slice type (rustc's LUB), with the empty array
// promoted; and a one-variant enum `let` pattern over a borrowing
// variant.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
enum E2<'a> {
    V { r: &'a D },
}
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn pick(n: u8, v: &[u8]) -> usize {
    let s = match n {
        0 => &[],
        _ => v,
    };
    let z = if n == 0 { v } else { &[7, 8] };
    s.len() * 10 + z.len()
}
fn main() {
    println!("{} {}", pick(0, &[1, 2]), pick(1, &[1, 2, 3]));
    let x = E2::V { r: &mk("t") };
    let E2::V { r } = x;
    println!("use {}", r.0);
}
