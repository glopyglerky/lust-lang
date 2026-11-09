// Dependency macro paths rustc 1.99 accepts: `util::m!` and `::util::m!`
// from the root, a module, a closure, and another macro.
macro_rules! h {
    () => {
        ::util::m!()
    };
}

mod b {
    pub fn f() -> i32 {
        util::m!() + ::util::m!()
    }
}

fn main() {
    let k = || util::m!();
    println!("{} {} {} {}", util::m!(), b::f(), k(), h!());
}
