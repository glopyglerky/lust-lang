// A block's items are in scope from the block's start, as in rustc: a
// static, const, fn, or thread_local! key declared in a block shadows a
// parameter or an outer `let` of the same name. A local declared inside
// the block shadows a block fn. Non-snake names on purpose.
#![allow(non_snake_case, non_upper_case_globals, unused_variables)]
use std::cell::Cell;

fn param_vs_static(X: u32) -> u32 {
    static X: u32 = 7;
    X
}

fn param_vs_fn(X: u32) -> u32 {
    fn X() -> u32 {
        9
    }
    X()
}

fn param_vs_key(X: u32) -> u32 {
    thread_local! { static X: Cell<u32> = const { Cell::new(11) }; }
    X.get()
}

fn main() {
    println!("{} {} {}", param_vs_static(1), param_vs_fn(1), param_vs_key(1));
    let x = 1u32;
    let y = {
        static x: u32 = 7;
        x
    };
    let z = {
        const x: u32 = 8;
        x + 1
    };
    println!("{} {} {}", x, y, z);
    // A local declared inside the block shadows the block's fn.
    fn f() -> u32 {
        1
    }
    let f = 5u32;
    println!("{}", f);
    // Sibling fn names do not stop a nested fn's own parameters.
    fn g(f: u32) -> u32 {
        f + 10
    }
    println!("{}", g(2));
    // A body fn of a name hides nothing from a fresh binding.
    match 6u32 {
        f => println!("bound {}", f),
    }
}
