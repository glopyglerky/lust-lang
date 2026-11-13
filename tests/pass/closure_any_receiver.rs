// R26: a captured `&dyn Any`, `Box<dyn Any>`, or `Rc<dyn Any>` used directly as the receiver
// of `Any`'s methods inside a closure (by reference, by mutable reference, by value).
use std::any::{Any, TypeId};
use std::rc::Rc;
fn msg(payload: &(dyn Any + Send)) -> String {
    let m = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or_default();
    m.to_string()
}
fn main() {
    let s = String::from("x");
    println!("{}", msg(&s));
    let mut b: Box<dyn Any> = Box::new(5u32);
    let mut bump = || {
        if let Some(x) = b.downcast_mut::<u32>() {
            *x += 1;
        }
    };
    bump();
    bump();
    let peek = || b.downcast_ref::<u32>().copied();
    let tid = || (*b).type_id() == TypeId::of::<u32>();
    println!("{:?} {} {}", peek(), b.is::<u32>(), tid());
    let take = move || b.downcast::<u32>().map(|x| *x).unwrap_or(0);
    println!("{}", take());
    let rc: Rc<dyn Any> = Rc::new(String::from("s"));
    let f = || rc.downcast_ref::<String>().map(|s| s.len());
    println!("{:?} {}", f(), Rc::strong_count(&rc));
}
