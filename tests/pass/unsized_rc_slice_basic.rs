use std::rc::Rc;
fn main() {
    let a: Rc<[i32]> = Rc::from(vec![3, 1, 2]);
    let b = a.clone();
    println!("{:?} {} {} {}", a, b[1], b.len(), Rc::strong_count(&a));
    let s: i32 = a.iter().sum();
    println!("{s} {:?}", a.first());
    let c: Rc<[i32]> = Rc::new([1, 2, 3]);
    println!("{:?} {}", c, a == c);
    let d: Rc<[String]> = Rc::from(&[String::from("a"), String::from("b")][..]);
    println!("{:?}", d);
}
