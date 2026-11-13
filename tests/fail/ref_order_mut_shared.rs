// expect: L0113
fn main() {
    let mut a: i64 = 5;
    let b: i64 = 6;
    println!("{}", &mut a < &b);
}
