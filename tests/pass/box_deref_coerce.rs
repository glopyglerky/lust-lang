// &Box<T> coerces to &T at call sites.
fn show(x: &i32) { println!("show {}", x); }
fn shout(s: &str) { println!("shout {}", s); }
fn main() {
    let b = Box::new(9);
    show(&b);
    let s = Box::new("hey".to_string());
    shout(&s);
    let r: &i32 = &b;
    println!("{}", r);
}
