// expect: L0306
// A returned `Ref<'_, T>` keeps its `RefCell` borrowed (R26).
use std::cell::{Ref, RefCell};
struct S { c: RefCell<Vec<i32>> }
impl S { fn get(&self) -> Ref<'_, Vec<i32>> { self.c.borrow() } }
fn main() {
    let r;
    {
        let s = S { c: RefCell::new(vec![1]) };
        r = s.get();
    }
    println!("{:?}", r);
}
