// expect: L0311
fn main() {
    let b = Box::new(vec![1, 2]);
    let r = &b;
    let v = *b;
    println!("{:?} {:?}", r, v);
}
