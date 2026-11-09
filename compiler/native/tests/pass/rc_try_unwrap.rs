// Rc::try_unwrap gives the value back only when unique.
use std::rc::Rc;
fn main() {
    let a = Rc::new("only".to_string());
    match Rc::try_unwrap(a) { Ok(s) => println!("got {}", s), Err(_) => println!("shared") }
    let b = Rc::new(5);
    let c = b.clone();
    match Rc::try_unwrap(b) { Ok(v) => println!("got {}", v), Err(r) => println!("shared {}", r) }
    println!("{}", c);
}
