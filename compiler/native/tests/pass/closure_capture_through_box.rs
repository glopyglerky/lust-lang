// Closures capture through `Box` derefs precisely (a builtin deref in
// rustc's place model), and stop at overloaded derefs (`Rc`, guards,
// `String`, `Vec`, `Cow`), borrowing the pointer.
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Debug)]
struct S { x: u32, y: u32, v: Vec<u32> }
fn main() {
    let mut b = Box::new(S { x: 1, y: 2, v: vec![1, 2, 3] });
    let i = 1usize;
    // `(*b).x` only: `b.y` may change while `f` lives.
    let f = || b.x + b.v[i];
    b.y += 1;
    println!("{} {:?}", f(), b);
    let mut g = || { b.x = 7; b.v.push(4); b.v.len() };
    println!("{} {:?}", g(), b);
    let mut arr = Box::new([1u32, 2, 3, 4]);
    let mut h = || { arr[i] += 5; arr[i] };
    println!("{} {:?}", h(), arr);
    let m = move || b.v.len() as u32 + b.x;
    println!("{}", m());
    let r = Rc::new([4u32, 5]);
    let k = || r[i] + r.len() as u32;
    println!("{} {:?}", k(), r);
    let c = RefCell::new(vec![1u32, 2]);
    let mut gd = c.borrow_mut();
    let mut q = || gd[0] = 7;
    q();
    drop(gd);
    println!("{:?}", c.borrow());
    let content = String::from("ab");
    let other = RefCell::new(String::from("ab"));
    let mut changed = true;
    let mut cmp = || { if *content == *other.borrow() { changed = false; } };
    cmp();
    println!("{} {}", changed, content);
    let names = vec![String::from("a"), String::from("b")];
    let ts = [Cow::Borrowed("b"), Cow::Owned(String::from("z"))];
    for t in &ts {
        println!("{}", names.iter().any(|x| x.as_str() == &**t));
    }
    let mut s = String::from("ab");
    let mut up = || { let r: &mut str = &mut *s; r.len() };
    println!("{} {}", up(), s);
}
