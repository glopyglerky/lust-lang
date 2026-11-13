// expect: L0312
// rustc E0506: a struct field `Option<Box<dyn Tr + 'a>>` and an
// `Rc<dyn Tr>` are owned trait objects: their drop uses their borrows.
use std::rc::Rc;
trait Tr {
    fn get(&self) -> i32;
}
struct P<'a>(&'a i32);
impl Tr for P<'_> {
    fn get(&self) -> i32 {
        *self.0
    }
}
struct H<'a> {
    f: Option<Box<dyn Tr + 'a>>,
}
fn main() {
    let mut x = 1;
    let h = H { f: Some(Box::new(P(&x))) };
    println!("{}", h.f.as_ref().unwrap().get());
    let mut y = 2;
    let r: Rc<dyn Tr + '_> = Rc::new(P(&y));
    println!("{}", r.get());
    y += 1;
    x += 1;
    println!("{} {}", x, y);
}
