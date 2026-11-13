// Ordering and core types compare by structure: Ordering itself, tuples,
// str and String, Vec (lexicographic, then length), Option (None first),
// Result (Ok first), float arrays (unordered on NaN), and Box.
use std::cmp::Ordering;
fn ordering() {
    let a = Ordering::Less; let b = Ordering::Greater;
    println!("{} {:?} {:?} {}", a < b, a.cmp(&b), a.partial_cmp(&b), a == Ordering::Less);
    println!("{:?}", (1, 'x').cmp(&(1, 'y')));
    println!("{:?}", "abc".partial_cmp("ab"));
    println!("{:?}", String::from("b").cmp(&String::from("ab")));
}
fn containers() {
    let a = vec![1, 2, 3]; let b = vec![1, 2];
    println!("{} {:?} {:?}", a < b, a.cmp(&b), a.partial_cmp(&b));
    let x: Option<u8> = None; let y = Some(0u8);
    println!("{} {:?}", x < y, x.cmp(&y));
    let r1: Result<u8, u8> = Ok(5); let r2: Result<u8, u8> = Err(0);
    println!("{} {:?}", r1 < r2, r1.cmp(&r2));
    println!("{:?}", [1.0, f64::NAN].partial_cmp(&[1.0, 2.0]));
    println!("{}", Box::new(3) > Box::new(2));
}
fn main() {
    ordering();
    containers();
}
