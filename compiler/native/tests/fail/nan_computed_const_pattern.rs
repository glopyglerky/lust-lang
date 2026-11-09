// expect: L0110
const F: f64 = f64::INFINITY - f64::INFINITY;
fn main() {
    match 1.0f64 {
        F => println!("f"),
        _ => println!("o"),
    }
}
