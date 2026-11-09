// R02 (second family): a value whose references are `'static` (a written `&'static`, a
// static item, a string literal) may go to a `'static` use; the borrow checker proves it.
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
struct Msg(i32);
static M: Msg = Msg(1);
type Data = (Option<u32>, Option<HashSet<&'static str>>);
fn take(n: u32) -> Data {
    let mut s = HashSet::new();
    s.insert("a");
    (Some(n), Some(s))
}
fn free(g: Box<dyn Send>) -> u32 {
    drop(g);
    1
}
fn mk() -> Box<dyn Fn() -> usize> {
    let reports: Rc<RefCell<Vec<&'static Msg>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = reports.clone();
    Box::new(move || {
        sink.borrow_mut().push(&M);
        sink.borrow().len()
    })
}
fn main() {
    let f = mk();
    f();
    println!("{}", f());
    let garbage = vec![take(1), take(2)];
    println!("{}", free(Box::new(garbage)));
    let s = "lit";
    let g: Box<dyn Fn() -> usize> = Box::new(move || s.len());
    println!("{}", g());
}
