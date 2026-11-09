// Box::new, deref reads and writes, Display and Debug forwarding.
fn main() {
    let b = Box::new(41);
    println!("{} {:?}", *b + 1, b);
    let s = Box::new("hi".to_string());
    println!("{} {}", s.len(), *s);
    let mut m = Box::new(1);
    *m += 5;
    println!("{}", m);
}
