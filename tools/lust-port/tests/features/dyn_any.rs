// feature: dyn.any
// form: Box<dyn Any> with downcast_ref, as in ts-rust protocol payloads
use std::any::Any;
fn show(v: &dyn Any) -> String {
    if let Some(n) = v.downcast_ref::<i32>() { format!("int {n}") } else { "other".to_string() }
}
fn main() { let b: Box<dyn Any> = Box::new(7i32); println!("{}", show(&*b)); }
