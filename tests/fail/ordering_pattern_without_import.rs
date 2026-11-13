// expect: L0102
// Ordering is not in the prelude, in patterns either.
fn main() {
    let o = 1.cmp(&2);
    match o {
        Ordering::Less => println!("lt"),
        _ => println!("other"),
    }
}
