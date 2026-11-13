// expect: L0110
fn main() {
    let mut x = 0;
    let o = Some(42);
    Some(x) = o;
    println!("{}", x);
}
