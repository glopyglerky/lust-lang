// expect: L0802
// rustc E0433: `super` at the crate root has no parent module.
#[macro_export]
macro_rules! m {
    () => {
        1
    };
}
fn main() {
    println!("{}", super::m!());
}
