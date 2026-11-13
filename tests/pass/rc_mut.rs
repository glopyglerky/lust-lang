// Rc::make_mut clones on write; Rc::get_mut only when unique.
use std::rc::Rc;
fn main() {
    let mut a = Rc::new(10);
    *Rc::make_mut(&mut a) += 1;
    println!("{}", a);
    let b = a.clone();
    *Rc::make_mut(&mut a) += 10;
    println!("{} {}", a, b);
    let mut c = Rc::new(3);
    match Rc::get_mut(&mut c) { Some(x) => *x = 9, None => {} }
    println!("{}", c);
    let d = c.clone();
    println!("{:?}", Rc::get_mut(&mut c));
    drop(d);
}
