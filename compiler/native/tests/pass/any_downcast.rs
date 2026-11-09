// core::any: is, downcast_ref, downcast_mut, Box downcast, type_id, TypeId::of (W-CHECK-G3).
use std::any::{Any, TypeId};
struct S(i32);
fn show(v: &dyn Any) -> String {
    if let Some(n) = v.downcast_ref::<i32>() { format!("int {n}") }
    else if let Some(s) = v.downcast_ref::<String>() { format!("string {s}") }
    else if v.is::<S>() { "S".to_string() }
    else { "other".to_string() }
}
fn main() {
    let b: Box<dyn Any> = Box::new(7i32);
    println!("{}", show(&*b));
    println!("{}", show(&String::from("hi")));
    println!("{}", show(&S(1)));
    println!("{}", show(&1.5f64));
    println!("{}", TypeId::of::<i32>() == TypeId::of::<i32>());
    println!("{}", TypeId::of::<i32>() == TypeId::of::<u32>());
    let x = 5u8;
    println!("{}", x.type_id() == TypeId::of::<u8>());
    println!("{}", (&x as &dyn Any).type_id() == TypeId::of::<u8>());
    println!("{}", b.type_id() == TypeId::of::<Box<dyn Any>>());
    println!("{}", (*b).type_id() == TypeId::of::<i32>());
    let mut m: Box<dyn Any> = Box::new(S(3));
    if let Some(s) = m.downcast_mut::<S>() { s.0 += 10; }
    println!("{}", m.downcast_ref::<S>().unwrap().0);
    match m.downcast::<S>() { Ok(s) => println!("ok {}", s.0), Err(_) => println!("err") }
    let e: Box<dyn Any + Send> = Box::new(String::from("s"));
    match e.downcast::<i32>() { Ok(_) => println!("int"), Err(e) => println!("still {}", e.is::<String>()) }
}
