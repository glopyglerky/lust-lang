// expect: L0802
// rustc E0432: `self::k` reads the module's namespace, where a
// `macro_rules!` definition is not.
mod m2 {
    macro_rules! k {
        () => {
            1u32
        };
    }
    pub(crate) use self::k;
}
fn main() {
    println!("{}", m2::k!());
}
