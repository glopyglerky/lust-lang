// expect: L0705
// rustc E0597: the impl is for `Option<Rc<dyn Logger + 'static>>`, and `o`'s inferred object
// lifetime then borrows a local.
use std::rc::Rc;
trait Logger { fn log(&self, msg: &str); }
struct L<'a>(&'a str);
impl Logger for L<'_> { fn log(&self, msg: &str) { println!("log {} {msg}", self.0); } }
impl Logger for Option<Rc<dyn Logger>> {
    fn log(&self, msg: &str) { if let Some(l) = self { l.log(msg); } }
}
fn log_to(l: &dyn Logger, m: &str) { l.log(m); }
fn main() {
    let s = String::from("x");
    let o: Option<Rc<dyn Logger>> = Some(Rc::new(L(&s)));
    log_to(&o, "out");
}
