// ARCH 2.9 and 2.10 step 3: typed receivers match by receiver
// type. `self: &Rc<Self>` takes a reference to the Rc value; it
// does not autoderef to Self first. ts-rust uses &Rc<Self> in
// frontend/packagejson.rs:1092 (159 sites per ARCH 2.9).
use std::rc::Rc;
use std::sync::Arc;
struct Entry {
    name: String,
}
impl Entry {
    fn same(self: &Rc<Self>, name: &str) -> bool {
        self.name == name
    }
    fn cloned(self: &Rc<Self>) -> Rc<Self> {
        Rc::clone(self)
    }
    fn owned_name(self: Rc<Self>) -> String {
        self.name.clone()
    }
    fn arc_len(self: &Arc<Self>) -> usize {
        self.name.len()
    }
    fn boxed_len(self: Box<Self>) -> usize {
        self.name.len()
    }
}
fn main() {
    let e = Rc::new(Entry { name: String::from("pkg") });
    let same = e.same("pkg");
    let copy = e.cloned();
    let owned = copy.owned_name();
    let a = Arc::new(Entry { name: String::from("tool") });
    let b = Box::new(Entry { name: String::from("box!") });
    println!("{same} {owned} {} {}", a.arc_len(), b.boxed_len());
}
