// Captured index and value variables used on a closure-local guard or
// smart pointer (ts-rust ast/store.rs `hot_list_of`, core.rs `text`).
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;
thread_local! { static L: RefCell<[u32; 4]> = const { RefCell::new([0; 4]) }; }
fn get(slot: usize, n: u32) -> Option<u32> {
    let hit = L.try_with(|l| { let e = l.borrow()[slot]; if e == n { Some(e) } else { None } }).ok().flatten();
    let _ = L.try_with(|l| { if let Ok(mut l) = l.try_borrow_mut() { l[slot] = n + 1; } });
    hit
}
fn main() {
    println!("{:?} {:?} {:?}", get(1, 0), get(1, 1), get(2, 5));
    let slots: Box<[OnceLock<&'static str>]> = vec![OnceLock::new(), OnceLock::new()].into_boxed_slice();
    let _ = slots[1].set("hi");
    let index = 1usize;
    println!("{:?}", Some(&slots).and_then(|s| s[index].get()).copied());
    let cell = RefCell::new(vec![1u32, 2, 3]);
    let i = 2usize;
    let bump = || { let mut v = cell.borrow_mut(); v[i] += 10; v[i] };
    println!("{} {}", bump(), bump());
    let rc: Rc<[u32]> = Rc::from(vec![4u32, 5, 6]);
    let peek = |r: &Rc<[u32]>| r[i] + r[..i].len() as u32;
    println!("{} {:?}", peek(&rc), cell.borrow());
}
