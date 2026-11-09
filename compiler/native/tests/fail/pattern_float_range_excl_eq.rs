// expect: L0119
fn main() {
    let x: f64 = 1.5;
    println!("{}", matches!(x, 1.5..1.5));
}
