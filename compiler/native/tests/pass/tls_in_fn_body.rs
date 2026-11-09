// thread_local! inside function bodies: each block has its own key, even
// with the same name, and the key lives for the thread, not the call.
use std::cell::Cell;

fn counter() -> u32 {
    thread_local! {
        static CALLS: Cell<u32> = const { Cell::new(0) };
    }
    CALLS.set(CALLS.get() + 1);
    CALLS.get()
}

fn other() -> u32 {
    thread_local! {
        static CALLS: Cell<u32> = Cell::new(100);
    }
    CALLS.set(CALLS.get() + 1);
    CALLS.get()
}

fn main() {
    println!("{} {} {}", counter(), counter(), counter());
    println!("{} {}", other(), other());
    println!("{}", counter());
}
