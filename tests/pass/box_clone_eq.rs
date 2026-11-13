// Box clone and equality follow the pointee.
fn main() {
    let a = Box::new(vec![1, 2]);
    let b = a.clone();
    println!("{:?} {:?} {}", a, b, a == b);
    let c = Box::new(5);
    let d = Box::new(6);
    println!("{}", c == d);
}
