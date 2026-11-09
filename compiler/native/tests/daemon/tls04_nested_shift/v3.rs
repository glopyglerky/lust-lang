use std::cell::Cell;
fn first() -> u32 {
    thread_local! {
        static C: Cell<u32> = const { Cell::new(0) };
    }
    C.set(C.get() + 1);
    C.get()
}
fn second() -> u32 {
    thread_local! {
        static C: Cell<u32> = const { Cell::new(100) };
    }
    C.set(C.get() + 1);
    C.get()
}
fn main() {
    println!("{} {} {}", first(), second(), first());
}
