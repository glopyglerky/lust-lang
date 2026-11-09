// expect: L0303
fn main() {
    let mut a: Option<String> = Some(String::from("x"));
    let r = a.as_ref();
    let t = a.take();
    println!("{:?} {:?}", r, t);
}
