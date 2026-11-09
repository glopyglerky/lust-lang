// expect: L0113
fn main() {
    let mut a: i64 = 5;
    let y: i64 = -&mut a;
    println!("{}", y);
}
