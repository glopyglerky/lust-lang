// expect: L0306
// rustc E0597: inside a closure, a call through `P: Fn(&'static u8)` still needs a `'static` borrow.
fn bad<P: Fn(&'static u8)>(p: &P) {
    let f = || {
        let x = 1u8;
        p(&x)
    };
    f()
}

fn main() {}
