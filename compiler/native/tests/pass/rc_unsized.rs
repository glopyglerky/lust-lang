use std::rc::Rc;
fn main() {
    let b: Rc<[i32]> = Rc::new([1, 2, 3]);
    println!("{:?}", b);
}
