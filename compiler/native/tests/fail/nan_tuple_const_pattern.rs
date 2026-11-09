// expect: L0110
const P: (f64, u8) = (f64::NAN, 1);
fn main() {
    match (1.0f64, 1u8) {
        P => println!("p"),
        _ => println!("o"),
    }
}
