use std::rc::Rc; use std::sync::Arc;
fn take(x: Rc<[i32]>) -> i32 { x.iter().sum() }
fn main() {
    let a: Box<[u8]> = Box::new([1, 2]);
    let b: Arc<[String]> = Arc::new([String::from("s"), String::from("t")]);
    println!("{:?} {:?} {}", a, b, take(Rc::new([4, 5, 6])));
    let c: Rc<[i32; 2]> = Rc::new([7, 8]);
    let d: Rc<[i32]> = c.clone();
    println!("{:?} {} {}", d, Rc::strong_count(&c), Rc::ptr_eq(&(c as Rc<[i32]>), &d));
}
