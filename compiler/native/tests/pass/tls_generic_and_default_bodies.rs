// A key inside a generic method is one key for every instance, and a key
// in a trait default body is one key for every impl, as in rustc.
use std::cell::Cell;
use std::fmt::Debug;
struct Wrap<T>(T);
impl<T: Debug> Wrap<T> {
    fn show(&self) -> String {
        thread_local! {
            static CALLS: Cell<u32> = const { Cell::new(0) };
        }
        CALLS.set(CALLS.get() + 1);
        format!("{:?} call {}", self.0, CALLS.get())
    }
}
trait Count {
    fn tick(&self) -> u32 {
        thread_local! {
            static TICKS: Cell<u32> = Cell::new(100);
        }
        TICKS.set(TICKS.get() + 1);
        TICKS.get()
    }
}
struct A;
struct B;
impl Count for A {}
impl Count for B {}
fn main() {
    println!("{}", Wrap(1u8).show());
    println!("{}", Wrap("x").show());
    println!("{}", Wrap(2u8).show());
    println!("{} {} {}", A.tick(), B.tick(), A.tick());
}
