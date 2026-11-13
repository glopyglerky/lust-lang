use std::cell::Cell;
thread_local! {
    static A: Cell<u32> = const { Cell::new(1) };
}
fn main() {
    A.set(A.get() + 10);
    println!("{}", A.get());
}
