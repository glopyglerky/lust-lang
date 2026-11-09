// expect: L0119
fn main() {
    const HI: f64 = 1.0;
    let x: f64 = 1.5;
    println!("{}", matches!(x, 2.0..=HI));
}
