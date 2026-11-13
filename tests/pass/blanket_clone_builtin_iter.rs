// A blanket `impl<T: Clone> Tag for T` on builtin iterators (Range,
// Chars, slice::Iter, adapters): as a method, by UFCS, and through
// generic fns with and without `?Sized`. The generic forms passed check
// and failed at build (L0612); the method form was L0104.
trait Tag {
    fn tag(&self) -> &'static str;
}
impl<T: Clone> Tag for T {
    fn tag(&self) -> &'static str {
        "blanket"
    }
}
fn g<U: Tag + ?Sized>(u: &U) -> &'static str {
    u.tag()
}
fn gv<U: Tag>(u: U) -> &'static str {
    u.tag()
}
fn main() {
    let x: std::ops::Range<i32> = 0..3;
    println!("{} {} {}", g(&x), x.tag(), Tag::tag(&x));
    println!("{}", gv(x));
    let c = "abc".chars();
    println!("{} {}", c.tag(), g(&c));
    let s = [1, 2].iter();
    println!("{} {}", s.tag(), g(&s));
    let r = (0..4).rev().enumerate();
    println!("{} {}", r.tag(), g(&r));
    let w = &(1..=3);
    println!("{}", w.tag());
}
