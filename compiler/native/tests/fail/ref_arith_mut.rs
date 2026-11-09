// expect: L0113
fn main() {
    let mut a: i64 = 5;
    let r = &mut a;
    let y: i64 = r + 1;
    println!("{}", y);
}
