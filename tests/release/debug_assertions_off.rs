// Release semantics (-C debug-assertions=off): debug_assert! forms do not run their checks,
// cfg(debug_assertions) is false, and a side effect inside a debug assertion is not evaluated.
use std::hint::black_box as bb;

#[cfg(debug_assertions)]
fn mode() -> &'static str {
    "debug"
}

#[cfg(not(debug_assertions))]
fn mode() -> &'static str {
    "release"
}

fn touch(n: &mut i32) -> bool {
    *n += 1;
    false
}

fn main() {
    let mut n = 0;
    debug_assert!(touch(&mut n), "not evaluated");
    debug_assert_eq!(bb(1), bb(2));
    debug_assert_ne!(bb(3), bb(3), "equal");
    println!("{} {} {}", mode(), cfg!(debug_assertions), n);
    if cfg!(debug_assertions) {
        println!("unreachable in release");
    }
}
