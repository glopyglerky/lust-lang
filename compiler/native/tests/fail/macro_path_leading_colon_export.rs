// expect: L0802
// rustc E0433: `::m` names an extern crate `m`, not the crate root macro.
#[macro_export]
macro_rules! m {
    () => {
        1
    };
}
fn main() {
    println!("{}", ::m!());
}
