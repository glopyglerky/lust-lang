// expect: L0630
const N: Option<u32> = '9'.to_digit(37);
fn main() {
    println!("{:?}", N);
}
