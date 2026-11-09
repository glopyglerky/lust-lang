// expect: L0524
#[inline]
struct S;

fn main() {
    let _s = S;
    println!("x");
}
