// `self: &Rc<Self>`, `self: Rc<Self>`, and `self: Box<Self>`
// receivers (ARCH 2.10 step 3), plus Rc::clone in assoc form.
use std::rc::Rc;
struct Entry { name: String }
impl Entry {
    fn same(self: &Rc<Self>, name: &str) -> bool { self.name == name }
    fn cloned(self: &Rc<Self>) -> Rc<Self> { Rc::clone(self) }
    fn owned_name(self: Rc<Self>) -> String { self.name.clone() }
    fn boxed_len(self: Box<Self>) -> usize { self.name.len() }
}
fn main() {
    let e = Rc::new(Entry { name: String::from("pkg") });
    let same = e.same("pkg");
    let copy = e.cloned();
    let owned = copy.owned_name();
    let b = Box::new(Entry { name: String::from("box!") });
    println!("{same} {owned} {}", b.boxed_len());
}
