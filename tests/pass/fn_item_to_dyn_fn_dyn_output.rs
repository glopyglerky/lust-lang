// A fn item coerced to `dyn Fn(..) -> R` where R holds a `dyn Tr`: the
// inner object's 'static lifetime pairs with the target's (R06).
use std::rc::Rc;
trait Tr { fn n(&self) -> i32; }
struct A;
impl Tr for A { fn n(&self) -> i32 { 1 } }
struct B<'a>(&'a i32);
impl Tr for B<'_> { fn n(&self) -> i32 { *self.0 } }
fn g2(_o: i32) -> Box<dyn Tr> { Box::new(A) }
fn g3(_o: i32) -> Option<Box<dyn Tr + 'static>> { Some(Box::new(A)) }
fn g4<'a>(x: &'a i32) -> Box<dyn Tr + 'a> { Box::new(B(x)) }
fn g5(_o: &str) -> Rc<dyn Tr> { Rc::new(A) }
fn call(f: &dyn Fn(i32) -> Box<dyn Tr>) -> i32 { f(1).n() }
fn call2(f: &dyn Fn(i32) -> Option<Box<dyn Tr>>) -> i32 { f(1).unwrap().n() }
fn call3(f: Box<dyn Fn(&i32) -> Box<dyn Tr + '_>>) -> i32 { let x = 5; f(&x).n() }
fn call4(f: &mut dyn FnMut(&str) -> Rc<dyn Tr>) -> i32 { let r; { let s = String::from("s"); r = f(&s); } r.n() }
fn main() {
    println!("{}", call(&g2));
    println!("{}", call(&|_x: i32| Box::new(A) as Box<dyn Tr>));
    println!("{}", call2(&g3));
    println!("{}", call3(Box::new(g4)));
    println!("{}", call4(&mut g5));
}
