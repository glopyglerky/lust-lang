// Fn items (plain and generic) and fn pointers as `dyn Fn` values: the
// vtable entry calls the fn (or the stored pointer), for `&dyn`, `Box`,
// `Rc`, and `Box<dyn FnOnce>`, a large return, and a mixed Vec.
use std::rc::Rc;
fn double(x: u32) -> u32 { x * 2 }
fn id<T>(t: T) -> T { t }
fn shout(s: &str) -> String { s.to_uppercase() }
#[derive(Debug)]
struct Big([u64; 6]);
fn big(n: u64) -> Big { Big([n; 6]) }
fn main() {
    let a: &dyn Fn(u32) -> u32 = &double;
    let b: Box<dyn Fn(u32) -> u32> = Box::new(double);
    let c: Rc<dyn Fn(u32) -> u32> = Rc::new(id::<u32>);
    let p: fn(u32) -> u32 = double;
    let d: Box<dyn Fn(u32) -> u32> = Box::new(p);
    println!("{} {} {} {}", a(1), b(2), c(3), d(4));
    let e: Box<dyn FnOnce(&str) -> String> = Box::new(shout);
    println!("{}", e("hi"));
    let f: Box<dyn FnOnce(u32) -> u32> = Box::new(p);
    println!("{}", f(5));
    let g: &dyn Fn(u64) -> Big = &big;
    println!("{:?}", g(7).0[5]);
    let hp: fn(u64) -> Big = big;
    let h: Box<dyn Fn(u64) -> Big> = Box::new(hp);
    println!("{:?}", h(8).0[0]);
    let fs: Vec<Box<dyn Fn(u32) -> u32>> = vec![Box::new(double), Box::new(|x| x + 1), Box::new(p)];
    println!("{:?}", fs.iter().map(|f| f(10)).collect::<Vec<_>>());
}
