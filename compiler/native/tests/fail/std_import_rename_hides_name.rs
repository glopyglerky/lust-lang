// expect: L0102
// A rename binds only the new name.
use std::rc::Rc as Foo;
fn main() {
    let a = Foo::new(1);
    let b = Rc::new(2);
    println!("{} {}", a, b);
}
