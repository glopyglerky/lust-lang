// expect: L0110
fn main() {
    match 1.0f64 {
        f64::NAN => println!("f"),
        _ => println!("o"),
    }
}
