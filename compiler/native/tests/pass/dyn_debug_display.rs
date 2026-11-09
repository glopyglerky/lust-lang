// `dyn Debug` and `dyn Display` objects (std fmt traits as dyn principals): the vtable formats
// through its header's fmt shim, with every format flag.
use std::fmt::{Debug, Display};
#[derive(Debug)]
struct P { x: i32, s: String }
struct D(u8);
impl Display for D { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "D<{}>", self.0) } }
fn show(items: &[&dyn Debug]) { for i in items { println!("{:?} | {:#?} | {:>8?}", i, i, i); } }
fn disp(items: &[&dyn Display]) { for i in items { println!("{} | {:>6} | {:<6}|", i, i, i); } }
fn boxed() -> Vec<Box<dyn Debug>> { vec![Box::new(1u8), Box::new("hi"), Box::new(vec![1, 2]), Box::new(Some(3.5f64))] }
fn f<'a>(x: &'a u8) -> &'a dyn Debug { x as &'a dyn Debug }
fn main() {
    let p = P { x: -3, s: "a\"b".into() };
    let t = (1u8, 'c', true);
    show(&[&5i32, &"str", &String::from("own"), &vec![Some(1), None], &p, &t, &1.25f32, &[1u8, 2, 3], &()]);
    disp(&[&7u64, &"x", &String::from("yy"), &D(9), &'z', &-2.5f64]);
    for b in boxed() { println!("{:?}", b); }
    println!("{:?}", f(&4));
    let d: &dyn Display = &D(1);
    let s = format!("[{}]", d);
    println!("{} {}", s, d.to_string());
}
