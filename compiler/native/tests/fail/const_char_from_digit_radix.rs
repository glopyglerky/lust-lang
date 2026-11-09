// expect: L0630
// from_digit panics above radix 36; in a const that is a reject in both.
const C: Option<char> = char::from_digit(3, 37);
fn main() {
    println!("{:?}", C);
}
