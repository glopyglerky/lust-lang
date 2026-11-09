// expect: L0113
fn main() {
    let mut a: i64 = 5;
    let r = &mut a;
    let mut y: i64 = 1;
    y += r;
    println!("{}", y);
}
