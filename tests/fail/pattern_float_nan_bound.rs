// expect: L0110
fn main() {
    let x: f64 = 1.5;
    println!("{}", matches!(x, 0.0..=f64::NAN));
}
