// expect: L0119
fn main() {
    let x: f64 = 1.5;
    println!("{}", matches!(x, 2.0..=1.0));
}
