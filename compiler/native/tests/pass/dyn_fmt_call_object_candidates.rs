// `.fmt(f)` on a dyn object inside a written fmt impl, with no fmt trait imported: rustc's
// object candidates (the principal's Debug/Display, or a supertrait's) need no import, and on a
// `Box<dyn U>` the impl's own trait (in scope inside the impl) is tried first at the Box step.
use std::fmt;
trait ShowS: fmt::Display {}
trait Both: fmt::Debug + fmt::Display {}
impl ShowS for u8 {}
impl Both for u8 {}
struct A(Box<dyn ShowS>);
impl fmt::Debug for A {
    // Box<dyn ShowS> is not Debug: the dyn step's Display::fmt.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
struct B(Box<dyn Both>);
impl fmt::Debug for B {
    // Both supertraits fit the dyn step, but the Box step's Debug wins.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
struct C<'a>(&'a dyn fmt::Display);
impl fmt::Debug for C<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        (*self.0).fmt(f)?;
        f.write_str("]")
    }
}
struct D(std::rc::Rc<dyn fmt::Debug>);
impl fmt::Display for D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
fn main() {
    println!("{:?} {:?} {:?} {}", A(Box::new(1)), B(Box::new(2)), C(&"s"), D(std::rc::Rc::new("q")));
    println!("{:>5?}|{:#?}|{:<4}|", A(Box::new(3)), C(&4.5), D(std::rc::Rc::new(vec![1])));
}
