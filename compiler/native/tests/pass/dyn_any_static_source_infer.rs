// ts-rust ast/source_file_ls.rs: `Rc<Cell<T>>` to `Rc<dyn Any>` with `T: 'static`, inside a
// closure where `Cell`'s type arg is still an inference variable at the coercion (it comes from
// the closure's return). lust decides "may hold a borrow" at body end.
use std::any::Any;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
struct Cell<T> { value: OnceCell<T> }
thread_local! { static DATA: RefCell<HashMap<u64, Rc<dyn Any>>> = RefCell::new(HashMap::new()); }
fn get<T: 'static>(k: u64) -> Rc<Cell<T>> {
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        if let Some(c) = data.get(&k) {
            return Rc::clone(c).downcast::<Cell<T>>().unwrap_or_else(|_| panic!("other type"));
        }
        let c = Rc::new(Cell { value: OnceCell::new() });
        let a: Rc<dyn Any> = c.clone();
        data.insert(k, a);
        c
    })
}
fn main() {
    let c = get::<u8>(1);
    let _ = c.value.set(7);
    println!("{:?} {:?}", get::<u8>(1).value.get(), get::<String>(2).value.get());
}
