// A macro's binder flips between the call site's name and its own ($x vs x): the call reads
// another binding, with no error (fn-body read sets and body hashes, 2026-10-20).
macro_rules! pick {
    ($x:ident) => {{
        let $x: u32 = 1;
        $x
    }};
}

fn f() -> u32 {
    let x: u32 = 5;
    pick!(x) + x
}

fn main() {
    println!("{}", f());
}
