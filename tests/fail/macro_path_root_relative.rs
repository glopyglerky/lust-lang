// expect: L0802
// rustc E0433: in 2018+ `a::m` resolves `a` in the current module (b), not the crate root.
mod a {
    macro_rules! m {
        () => {
            1
        };
    }
    pub(crate) use m;
}
mod b {
    pub fn f() {
        println!("{}", a::m!());
    }
}
fn main() {
    b::f();
}
