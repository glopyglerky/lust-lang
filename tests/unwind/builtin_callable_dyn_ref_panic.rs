// A `&dyn Fn` that panics inside `catch_unwind`: the panic is caught and
// the reference needs no drop.
use std::panic::{AssertUnwindSafe, catch_unwind};

fn main() {
    let msg = String::from("boom");
    let p: &dyn Fn() -> i32 = &|| panic!("{msg}");
    let caught = catch_unwind(AssertUnwindSafe(p));
    println!("{}", caught.is_err());
    let ok: &dyn Fn() -> i32 = &|| msg.len() as i32;
    println!("{:?}", catch_unwind(AssertUnwindSafe(ok)).ok());
}
