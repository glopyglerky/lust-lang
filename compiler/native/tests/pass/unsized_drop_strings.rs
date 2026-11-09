use std::rc::Rc; use std::sync::Arc;
fn main() {
    let v: Vec<String> = (0..5).map(|i| format!("s{i}")).collect();
    let r: Rc<[String]> = Rc::from(v);
    let r2 = r.clone();
    let a: Arc<[String]> = Arc::from(&r[..2]);
    drop(r);
    println!("{:?} {:?}", r2, a);
    let bs: Box<[String]> = vec![String::from("x")].into_boxed_slice();
    let ar: Arc<[String]> = Arc::from(bs);
    println!("{:?}", ar);
}
