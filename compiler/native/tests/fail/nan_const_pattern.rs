// expect: L0110
const F: f64 = f64::NAN;
fn main() {
    match 1.0f64 {
        F => println!("f"),
        _ => println!("o"),
    }
}
