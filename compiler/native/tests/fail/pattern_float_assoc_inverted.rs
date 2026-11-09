// expect: L0119
fn main() {
    let x: f64 = 1.5;
    println!("{}", matches!(x, f64::MAX..=0.0));
}
