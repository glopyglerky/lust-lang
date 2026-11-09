// expect: L0641
// A std conversion named through its trait has no fn item to
// reify into a function pointer in lust yet.
fn main() {
    let f: fn(u8) -> u32 = Into::into;
    println!("{}", f(5));
}
