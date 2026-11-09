use std::cell::Cell;
thread_local! {
    static Z: Cell<u32> = const { Cell::new(100) };
    static A: Cell<u32> = const { Cell::new(1) };
}
fn main() {
    A.set(A.get() + 10);
    Z.set(Z.get() + A.get());
    println!("{} {}", A.get(), Z.get());
}
