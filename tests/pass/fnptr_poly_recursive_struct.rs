// A struct that names itself (at other args) inside a fn pointer type
// held by value: the fn pointer typedef needs no complete struct, so the C
// type order has no cycle (was "field has incomplete type" in tcc).
// Polymorphic recursion rustc accepts: behind a reference, a fn pointer,
// ManuallyDrop, or a dyn type the drop check stops; a type with no drop
// glue held with a borrowed argument is never drop-checked; Rc nested 64
// deep is exactly at the limit.
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
struct ByRef<'a, T> { v: T, g: Option<&'a ByRef<'a, (T, T)>> }
struct ByFn<T> { v: T, g: Option<fn(ByFn<(T, T)>)> }
struct ByMd<T> { v: T, g: ManuallyDrop<Option<Box<ByMd<(T, T)>>>> }
struct ByDyn<T> { v: T, g: Option<Box<dyn Fn(ByDyn<(T, T)>)>> }
struct Ph<T> { v: T, p: PhantomData<Ph<Option<T>>> }
#[allow(dead_code)]
fn rc64(_x: std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<std::rc::Rc<u8>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>) {}
fn main() {
    let r = ByRef { v: 1u8, g: None };
    let f = ByFn { v: 2u8, g: None };
    let m = ByMd { v: 3u8, g: ManuallyDrop::new(None) };
    let d = ByDyn { v: 4u8, g: None };
    let x = 5u8;
    let p = Ph { v: &x, p: PhantomData };
    println!("{} {} {} {} {} {} {} {}", r.v, f.v, m.v, d.v, p.v, r.g.is_none(), f.g.is_none(), d.g.is_none());
}
