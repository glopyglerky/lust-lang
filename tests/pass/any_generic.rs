// Generic T: Any helpers and TypeId::of over generic and unsized types (W-CHECK-G3).
use std::any::Any;
fn get<T: Any>(a: &dyn Any) -> Option<&T> { a.downcast_ref::<T>() }
fn tid<T: 'static>() -> std::any::TypeId { std::any::TypeId::of::<T>() }
struct W<T>(T);
fn main() {
    let x = 7u16;
    println!("{:?} {:?}", get::<u16>(&x), get::<u32>(&x));
    println!("{} {}", tid::<W<u8>>() == tid::<W<u8>>(), tid::<W<u8>>() == tid::<W<u16>>());
    println!("{}", tid::<&str>() == std::any::TypeId::of::<&str>());
    let w = W(String::from("z"));
    let a: &dyn Any = &w;
    println!("{}", a.downcast_ref::<W<String>>().unwrap().0);
}
