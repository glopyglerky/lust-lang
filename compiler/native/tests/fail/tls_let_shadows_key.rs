// expect: L0110
// rustc E0308 (the key is a const there): a `let` cannot rebind the name
// of a thread_local! key declared in an enclosing block.
#![allow(non_snake_case)]
use std::cell::Cell;
fn main() {
    thread_local! { static X: Cell<u32> = const { Cell::new(7) }; }
    {
        let X = 1u32;
        println!("{}", X);
    }
    println!("{}", X.get());
}
