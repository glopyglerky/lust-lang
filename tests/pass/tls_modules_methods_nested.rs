// Keys in modules, impl and trait default bodies, a generic fn, nested
// `with` calls, and a cfg-selected key.
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
mod counters {
    use std::cell::Cell;
    thread_local! {
        pub static HITS: Cell<u32> = const { Cell::new(0) };
        pub(crate) static NAMES: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
    }
    pub fn hit() -> u32 {
        HITS.set(HITS.get() + 1);
        HITS.get()
    }
}
thread_local! {
    static OUTER: RefCell<Vec<u32>> = RefCell::new(Vec::new());
    #[cfg(unix)]
    static ONLY_UNIX: u8 = 1;
    #[cfg(windows)]
    static ONLY_UNIX: u8 = 2;
}
struct S { base: u32 }
impl S {
    fn bump(&self) -> u32 {
        counters::HITS.with(|h| h.get() + self.base)
    }
}
trait Tr {
    fn tagged(&self) -> String {
        counters::NAMES.with_borrow(|v| format!("{} names", v.len()))
    }
}
impl Tr for S {}
fn show<T: Debug>(x: T) -> String {
    OUTER.with_borrow_mut(|v| v.push(1));
    let n = OUTER.with_borrow(|v| v.len());
    format!("{:?} {}", x, n)
}
fn main() {
    println!("{} {}", counters::hit(), counters::hit());
    counters::NAMES.with_borrow_mut(|v| v.push("x".to_string()));
    let s = S { base: 10 };
    println!("{} {}", s.bump(), s.tagged());
    println!("{} {}", show(5u8), show("a"));
    OUTER.with(|o| {
        counters::HITS.with(|h| {
            o.borrow_mut().push(h.get());
            println!("nested {:?}", o.borrow());
        })
    });
    ONLY_UNIX.with(|u| println!("unix {}", u));
    let c = Cell::new(3);
    println!("{}", c.get());
}
