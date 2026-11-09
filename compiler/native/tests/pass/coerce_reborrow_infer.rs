// Coercing `&&T` to `&?X` reborrows first (`?X = &T`), as rustc tries
// the fewest derefs first; a deref coercion happens only when the
// reborrow does not fit.
trait Wrap { fn w(&self) -> u8; }
struct S;
impl Wrap for S { fn w(&self) -> u8 { 20 } }
impl<T: Wrap + ?Sized> Wrap for &T { fn w(&self) -> u8 { (**self).w() + 1 } }
fn via_bound<X: Wrap>(v: &X) -> u8 { v.w() }
fn size<X>(_v: &X) -> usize { std::mem::size_of::<X>() }
fn main() {
    let s = S;
    let r = &s;
    println!("{} {}", via_bound(&r), via_bound(r));
    println!("{} {}", size(&&5u8), size(&5u8));
}
