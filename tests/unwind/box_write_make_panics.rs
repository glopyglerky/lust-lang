// make() panics between Box::new_uninit and Box::write: the uninit box
// frees with no drop of T, and the value made before the panic drops once.
use std::cell::Cell;
use std::panic;
thread_local! { static DROPS: Cell<u32> = Cell::new(0); }
struct D(u32);
impl Drop for D { fn drop(&mut self) { DROPS.with(|d| d.set(d.get() + 1)); } }
fn make_box<T>(make: impl FnOnce() -> T) -> Box<T> { Box::write(Box::new_uninit(), make()) }
fn main() {
    panic::set_hook(Box::new(|_| {}));
    let r = panic::catch_unwind(|| make_box(|| -> D { let _x = D(1); panic!("boom") }));
    println!("{} {}", r.is_err(), DROPS.with(|d| d.get()));
    let r = panic::catch_unwind(|| make_box(|| D(2)).0);
    println!("{:?} {}", r.ok(), DROPS.with(|d| d.get()));
}
