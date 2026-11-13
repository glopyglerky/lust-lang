use std::rc::Rc;
fn main() {
    let mut a: Rc<str> = Rc::from("abc");
    { let m: &mut str = Rc::make_mut(&mut a); let _ = m.len(); }
    let b = a.clone();
    let _ = Rc::make_mut(&mut a).len();
    println!("{a} {b} {} {}", Rc::strong_count(&a), Rc::strong_count(&b));
    let mut s: Rc<[String]> = Rc::from(vec![String::from("x")]);
    let t = s.clone();
    Rc::make_mut(&mut s)[0].push('y');
    println!("{:?} {:?}", s, t);
    println!("{:?} {:?}", Rc::get_mut(&mut s).map(|v| v.len()), Rc::get_mut(&mut a).is_some());
    let c = a.clone();
    println!("{:?}", Rc::get_mut(&mut a).is_some());
    drop(c);
}
