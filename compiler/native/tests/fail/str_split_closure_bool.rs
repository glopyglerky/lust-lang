// expect: L0110
// rustc E0271: a split predicate returns bool.
fn main() {
    println!("{:?}", "a,b".split(|c: char| c as u32).count());
}
