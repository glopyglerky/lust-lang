// expect: L0630
// The unsuffixed literal falls back to i32 inside the shift, so it
// is out of range before the cast to i64.
const X: i64 = (2147483648 >> 7) as i64;

fn main() {
    println!("{}", X);
}
