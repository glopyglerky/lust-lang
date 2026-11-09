use std::cell::Cell;
thread_local! {
    static N: Cell<u32> = {
        println!("init");
        Cell::new(50)
    };
}
fn bump() -> u32 {
    N.set(N.get() + 1);
    N.get()
}
fn main() {
    println!("{}", bump());
    println!("{}", bump());
}
