use std::rc::Rc;
fn main() { let a: Rc<str> = Rc::from("abc"); let b: Rc<str> = Rc::from("abd"); println!("{} {} {} {}", a == b, a < b, a != b, *a == *"abc"); println!("{} {:?}", a.as_ref() == "abc", a.cmp(&b)); let x: Box<[i32]> = vec![1].into_boxed_slice(); let y: Rc<[i32]> = Rc::from(vec![1]); println!("{}", *x == *y); }
