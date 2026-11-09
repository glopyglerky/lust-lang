// expect: L0630
// is_alphabetic is not a const fn in rustc, so both reject it in a const.
const A: bool = 'é'.is_alphabetic();
fn main() {
    println!("{}", A);
}
