// rc::Weak and sync::Weak of str and [T] (ts-rust ast/file_version.rs FileText::weak).
use std::rc::{self, Rc};
use std::sync::{self, Arc};
#[derive(Debug)]
struct D(u8);
impl Drop for D { fn drop(&mut self) { println!("drop D{}", self.0); } }
fn main() {
    let a: Rc<str> = Rc::from("hello");
    let w: rc::Weak<str> = Rc::downgrade(&a);
    let w2 = w.clone();
    println!("{} {} {:?}", Rc::strong_count(&a), Rc::weak_count(&a), w);
    println!("{:?} {} {}", w.upgrade(), w.strong_count(), w.weak_count());
    println!("{}", w.ptr_eq(&w2));
    drop(a);
    println!("{:?} {} {}", w.upgrade(), w.strong_count(), w.weak_count());
    let s: Arc<[D]> = Arc::from(vec![D(1), D(2)]);
    let ws: sync::Weak<[D]> = Arc::downgrade(&s);
    println!("{:?}", ws.upgrade().map(|v| v.len()));
    let s2 = s.clone();
    drop(s);
    println!("{} {}", ws.strong_count(), ws.weak_count());
    drop(s2);
    println!("after {:?} {}", ws.upgrade().is_none(), ws.weak_count());
    let e: Arc<str> = Arc::from("");
    let we = Arc::downgrade(&e);
    println!("{:?}", we.upgrade());
    let v: Vec<rc::Weak<[u16]>> = { let r: Rc<[u16]> = Rc::from(vec![1u16, 2, 3]); vec![Rc::downgrade(&r), Rc::downgrade(&r)] };
    println!("{:?}", v.iter().map(|w| w.upgrade().is_some()).collect::<Vec<_>>());
}
