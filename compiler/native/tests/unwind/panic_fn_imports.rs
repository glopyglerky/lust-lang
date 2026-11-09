// catch_unwind, resume_unwind, and panic_any imported by name, renamed,
// and through `self`.
use std::panic::{self, catch_unwind as cu, panic_any, resume_unwind};

fn main() {
    let n = std::env::args().count();
    let r = cu(|| 1);
    println!("{:?}", r.ok());
    let r = panic::catch_unwind(|| {
        if n < 9 {
            resume_unwind(Box::new(5u8));
        }
    });
    println!("{:?}", r.err().and_then(|p| p.downcast_ref::<u8>().copied()));
    let r = cu(|| {
        if n < 9 {
            panic_any(7i32);
        }
    });
    println!("{:?}", r.err().and_then(|p| p.downcast_ref::<i32>().copied()));
}
