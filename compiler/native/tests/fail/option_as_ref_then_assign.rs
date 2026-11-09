// expect: L0312
fn main() {
    let mut a: Option<String> = Some(String::from("x"));
    let r = a.as_ref();
    a = None;
    println!("{:?} {:?}", r, a);
}
