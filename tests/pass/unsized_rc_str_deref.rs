use std::rc::Rc;
fn takes(s: &str) -> usize { s.len() }
fn main() {
    let a: Rc<str> = Rc::from("hello");
    println!("{}", takes(&a));
    let r: &str = &a;
    let t: &str = &*a;
    println!("{r} {t} {}", a.as_ref() as &str);
    let parts: Vec<&str> = a.split('l').collect();
    println!("{:?}", parts);
    let v: Rc<[u8]> = Rc::from(vec![1u8, 2]);
    let sl: &[u8] = &v;
    println!("{:?} {:?}", sl, &v[..1]);
}
