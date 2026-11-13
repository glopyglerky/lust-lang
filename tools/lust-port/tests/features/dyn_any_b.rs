// feature: dyn.any
// form: panic-payload shapes (&(dyn Any + Send), Box<dyn Any + Send>::downcast), Any as a supertrait with as_any
use std::any::Any;
trait Value: Any { fn as_any(&self) -> &dyn Any; }
struct Num(i64);
impl Value for Num { fn as_any(&self) -> &dyn Any { self } }
fn message(p: &(dyn Any + Send)) -> String {
    if let Some(s) = p.downcast_ref::<&str>() { s.to_string() } else if let Some(s) = p.downcast_ref::<String>() { s.clone() } else { "?".to_string() }
}
fn main() {
    let b: Box<dyn Any + Send> = Box::new(String::from("boom"));
    println!("{}", message(&*b));
    match b.downcast::<String>() { Ok(s) => println!("{}", s), Err(_) => println!("no") }
    let v: Box<dyn Value> = Box::new(Num(7));
    if let Some(n) = v.as_any().downcast_ref::<Num>() { println!("{}", n.0); }
}
