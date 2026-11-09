// expect: L0102
// Ordering is not in the prelude.
fn main() {
    let o = 1.cmp(&2);
    println!("{}", o == Ordering::Less);
}
