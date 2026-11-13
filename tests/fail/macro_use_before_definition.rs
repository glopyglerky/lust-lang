// expect: L0506
// rustc E0432: `use k;` reads textual scope at the `use`, and the
// definition comes after it.
mod m2 {
    pub(crate) use k;
    macro_rules! k {
        () => {
            1u32
        };
    }
}
fn main() {
    println!("{}", m2::k!());
}
