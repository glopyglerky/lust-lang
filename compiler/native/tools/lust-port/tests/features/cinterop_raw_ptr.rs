// feature: cinterop.raw_ptr
// form: raw pointers as identity values: coercion, from_ref, cast, addr, HashMap keys
use std::collections::HashMap;
use std::rc::Rc;
fn main() {
    let a = Rc::new(1i32);
    let b = a.clone();
    let mut m: HashMap<*const i32, i32> = HashMap::new();
    m.insert(Rc::as_ptr(&a), 7);
    let p: *const () = std::ptr::from_ref(&*b).cast();
    let n: *const i32 = std::ptr::null();
    let key = Rc::as_ptr(&b);
    println!("{} {} {} {}", m[&key], p.addr() == Rc::as_ptr(&a).addr(), n.is_null(), std::ptr::eq(&*a, &*b));
}
