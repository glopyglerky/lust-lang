// expect: L0630
// The literal pins to i8 from `i8::MIN`, and the product overflows.
fn main() {
    let v = const { 3 * i8::MIN };
    println!("{}", v);
}
