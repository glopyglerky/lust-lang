// expect: L0802
// rustc: a `use` in a module does not bind the name in a nested module.
mod m2 {
    macro_rules! k {
        () => {
            2u32
        };
    }
    pub(crate) use k;
}
use m2::k;
mod a {
    pub fn f() -> u32 {
        k!()
    }
}
fn main() {
    println!("{}", a::f());
}
