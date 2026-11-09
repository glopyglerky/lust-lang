// expect: L0306
// rustc E0597: the field's `Rc<dyn Speak>` is `'static`, so `Rc::from` needs a
// `'static` box, but the box borrows `s` (the fn value form, as ts-rust's
// `npm_install.map(Arc::from)`).
use std::rc::Rc;
trait Speak {
    fn speak(&self) -> String;
}
struct Borrowed<'a>(&'a str);
impl Speak for Borrowed<'_> {
    fn speak(&self) -> String {
        self.0.to_string()
    }
}
struct Holder {
    f: Option<Rc<dyn Speak>>,
}
fn main() {
    let s = String::from("x");
    let b: Box<dyn Speak + '_> = Box::new(Borrowed(&s));
    let h = Holder { f: Some(b).map(Rc::from) };
    println!("{}", h.f.unwrap().speak());
}
