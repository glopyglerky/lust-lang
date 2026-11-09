// A trait method call whose impl depends on a type the call cannot fix
// yet is typed by the trait's signature; the impl is picked once an
// argument, the expected type, or the literal fallback fixes that type.
trait Tr { fn m(self) -> u32; }
impl Tr for u8 { fn m(self) -> u32 { 8 } }
impl Tr for i32 { fn m(self) -> u32 { 32 } }
trait Put<T> { fn put(&self, t: T) -> u32; }
struct S(u8);
impl Put<u8> for S { fn put(&self, _t: u8) -> u32 { 8 } }
impl Put<i32> for S { fn put(&self, _t: i32) -> u32 { 32 } }
trait Conv<T> { fn conv(&self) -> T; }
impl Conv<u16> for S { fn conv(&self) -> u16 { self.0 as u16 + 1000 } }
impl Conv<u32> for S { fn conv(&self) -> u32 { self.0 as u32 + 100000 } }
struct M(u32);
impl From<u8> for M { fn from(x: u8) -> M { M(x as u32 + 1) } }
impl From<u16> for M { fn from(x: u16) -> M { M(x as u32 + 2) } }
struct N(u32);
impl From<u16> for N { fn from(x: u16) -> N { N(x as u32 + 3) } }
struct W<T>(Option<T>);
impl<T> W<T> {
    fn new() -> W<T> { W(None) }
    fn n(&self) -> u8 { 1 }
}
fn main() {
    let x = 5;
    let y = 6u8;
    println!("{} {} {} {}", x.m(), y.m(), S(0).put(3), S(0).put(3u8));
    let s = S(5);
    let a: u16 = s.conv();
    let b: u32 = s.conv();
    let later = s.conv();
    let fixed: u16 = later;
    println!("{} {} {}", a, b, fixed);
    let c = M::from(1u8);
    let d = M::from(1u16);
    let e = N::from(1);
    let w = W::<u8>::new();
    println!("{} {} {} {}", c.0, d.0, e.0, w.n());
}
