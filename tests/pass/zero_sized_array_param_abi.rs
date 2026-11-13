// Zero-sized by-value parameters ([T; 0], nested forms, a ZST with Drop,
// fn pointers, closures, a dyn method) must not shift the other arguments.
struct Dz;
impl Drop for Dz { fn drop(&mut self) { println!("drop Dz"); } }
fn a(v: [u8; 0], x: u32) -> u32 { x + v.len() as u32 }
fn b(x: u32, v: [[u16; 0]; 3], y: u32) -> u32 { x + y + v.len() as u32 }
fn c(v: ([u8; 0], u32), y: u32) -> u32 { v.1 + y }
fn d(v: (), y: u32) -> u32 { let _ = v; y }
fn e(v: [Dz; 0], y: u32) -> u32 { let _ = &v; y }
fn f(v: Dz, y: u32) -> u32 { let _ = &v; y }
fn g(y: u32) -> [u8; 0] { let _ = y; [] }
fn h(y: u32) -> (u32, [u8; 0]) { (y + 1, []) }
trait T { fn m(&self, v: [u8; 0], y: u32) -> u32; }
struct S;
impl T for S { fn m(&self, _v: [u8; 0], y: u32) -> u32 { y * 2 } }
fn main() {
    println!("{}", a([], 5));
    println!("{}", b(1, [[], [], []], 2));
    println!("{}", c(([], 10), 3));
    println!("{}", d((), 4));
    println!("{}", e([], 6));
    println!("{}", f(Dz, 7));
    println!("{:?} {:?}", g(9), h(9));
    let fp: fn([u8; 0], u32) -> u32 = a;
    println!("{}", fp([], 11));
    let cl = |v: [u8; 0], y: u32| y + v.len() as u32 + 100;
    println!("{}", cl([], 1));
    let cap = 5u32;
    let cl2 = move |v: [u8; 0], y: u32| cap + y + v.len() as u32;
    println!("{}", cl2([], 2));
    let t: &dyn T = &S;
    println!("{}", t.m([], 21));
    let bx: Box<dyn FnOnce([u8; 0], u32) -> u32> = Box::new(|_v: [u8; 0], y: u32| y + 1);
    println!("{}", bx([], 41));
    println!("end");
}
