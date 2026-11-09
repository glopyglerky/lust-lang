// expect: L0630
// `1 << 40` at the i32 fallback overflows the shift (rustc E0080).
const X: isize = ((1 << 40) as i8) as isize;

fn main() {
    println!("{}", X);
}
