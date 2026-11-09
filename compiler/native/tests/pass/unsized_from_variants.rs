use std::rc::Rc; use std::sync::Arc;
fn main() {
    let bs: Box<str> = Box::from("bx");
    let r: Rc<str> = Rc::from(bs);
    let bsl: Box<[i32]> = vec![1, 2].into_boxed_slice();
    let a: Arc<[i32]> = Arc::from(bsl);
    let arr: Rc<[u8]> = Rc::from([9u8, 8]);
    let m: Box<[i32]> = Box::from(&[5, 6][..]);
    let n: Box<[i32]> = Box::from([7, 8]);
    let s = String::from(Box::<str>::from("zz"));
    println!("{r} {:?} {:?} {:?} {:?} {s}", a, arr, m, n);
}
