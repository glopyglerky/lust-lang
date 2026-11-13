// rustc E0433: a dependency is not an item of the crate root without a
// `use util;` there, so `crate::util::m!` finds nothing.
fn main() {
    println!("{}", crate::util::m!());
}
