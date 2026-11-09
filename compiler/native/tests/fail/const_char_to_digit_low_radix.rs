// expect: L0630
// to_digit panics below radix 2; in a const that is a reject in both.
const N: Option<u32> = '9'.to_digit(1);
fn main() {
    println!("{:?}", N);
}
