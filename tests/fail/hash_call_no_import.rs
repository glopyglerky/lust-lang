// expect: L0104
// rustc E0599: `Hash::hash` needs the trait in scope.
fn main() {
    let mut d = std::hash::DefaultHasher::new();
    5u8.hash(&mut d);
    println!("{}", 1);
}
