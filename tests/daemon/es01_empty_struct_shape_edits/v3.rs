// The brace form with brace uses only; a lone `Z` in a pattern is a
// fresh binding.
#[derive(Debug)]
struct Z {}
fn main() {
    let z = Z {};
    let Z {} = z;
    let Z = 5;
    println!("{:?} {}", z, Z);
}
