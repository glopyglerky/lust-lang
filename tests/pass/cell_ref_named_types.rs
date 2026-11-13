// `Ref` and `RefMut` as named types (TRIAGE R26): returned from
// methods and fns with elided and named lifetimes, inside Option, and
// over a `dyn` element (ts-rust's printer `writer()` returns
// `RefMut<'_, dyn EmitTextWriter>`).
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

trait Writer {
    fn write(&mut self, s: &str);
    fn text(&self) -> String;
}

struct Buf(String);

impl Writer for Buf {
    fn write(&mut self, s: &str) {
        self.0.push_str(s);
    }
    fn text(&self) -> String {
        self.0.clone()
    }
}

struct Printer {
    writer: Option<Rc<RefCell<dyn Writer>>>,
    data: Option<RefCell<Vec<i32>>>,
}

impl Printer {
    fn writer(&self) -> RefMut<'_, dyn Writer> {
        self.writer.as_ref().expect("nil writer").borrow_mut()
    }
    fn data(&self) -> Option<Ref<'_, Vec<i32>>> {
        self.data.as_ref().map(|c| c.borrow())
    }
    fn mapper(&self) -> std::cell::Ref<'_, Vec<i32>> {
        self.data.as_ref().unwrap().borrow()
    }
}

fn show(r: Ref<i32>) -> i32 {
    *r + 1
}

fn bump(mut r: RefMut<i32>) {
    *r += 10;
}

fn first<'a, T>(c: &'a RefCell<Vec<T>>) -> Ref<'a, Vec<T>> {
    c.borrow()
}

fn main() {
    let p = Printer { writer: Some(Rc::new(RefCell::new(Buf(String::new())))), data: Some(RefCell::new(vec![1, 2])) };
    p.writer().write("hi ");
    p.writer().write("there");
    println!("{} {:?} {}", p.writer().text(), p.data().map(|d| d.len()), p.mapper()[1]);
    let c = RefCell::new(5);
    println!("{}", show(c.borrow()));
    bump(c.borrow_mut());
    println!("{} {:?}", c.borrow(), c.borrow());
    let v = RefCell::new(vec!["a", "b"]);
    println!("{:?} {}", first(&v), first(&v).len());
}
