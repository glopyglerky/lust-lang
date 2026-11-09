// expect: L0802
// rustc E0433: `::a` names an extern crate, never a module of this crate.
mod a {
    macro_rules! m {
        () => {
            1
        };
    }
    pub(crate) use m;
}
fn main() {
    println!("{}", ::a::m!());
}
