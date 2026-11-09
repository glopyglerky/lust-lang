// expect: L0001
// U+037A is Alphabetic but not XID_Continue, so it cannot be in a name.
fn main() {
    let aͺb = 1;
    println!("{}", aͺb);
}
