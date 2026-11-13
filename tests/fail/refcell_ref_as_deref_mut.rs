// expect: L0612
// rustc E0277: `Ref` has no `DerefMut`, so `Option<Ref<T>>::as_deref_mut` does not apply.
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(5u32);
    let mut o = Some(c.borrow());
    if let Some(v) = o.as_deref_mut() {
        *v += 1;
    }
}
