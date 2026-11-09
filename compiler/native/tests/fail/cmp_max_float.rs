// expect: L0612
// rustc E0277: f64 is not Ord, so std::cmp::max does not apply.
fn main() {
    println!("{}", std::cmp::max(1.5f64, 2.5));
}
