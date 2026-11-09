// expect: L0303
// rustc E0502: the closure holds a mutable borrow of `b` (its `downcast_mut` receiver) while
// `b` is borrowed again before the closure's last call.
use std::any::Any;
fn main() {
    let mut b: Box<dyn Any> = Box::new(5u32);
    let mut bump = || {
        if let Some(x) = b.downcast_mut::<u32>() {
            *x += 1;
        }
    };
    let r = b.is::<u32>();
    bump();
    println!("{}", r);
}
