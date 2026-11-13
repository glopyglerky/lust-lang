// User trait methods named like builtin std methods: a generic body
// calls the trait method, never the builtin method of the concrete type.
trait MyPow { fn pow(self, e: u32) -> Self; }
impl MyPow for u32 { fn pow(self, e: u32) -> u32 { self + e } }
trait L { fn len(&self) -> usize; }
impl L for String { fn len(&self) -> usize { 42 } }
trait P { fn push(&mut self, x: u8); }
impl P for Vec<u8> { fn push(&mut self, x: u8) { let _ = x; } }
trait TS { fn to_string(&self) -> String; }
impl TS for u8 { fn to_string(&self) -> String { String::from("ts") } }
trait U { fn unwrap(self) -> u8; }
impl U for Option<u8> { fn unwrap(self) -> u8 { 200 } }
trait Mn { fn min(self, o: Self) -> Self; }
impl Mn for i32 { fn min(self, o: i32) -> i32 { self + o } }
fn pw<T: MyPow>(t: T) -> T { t.pow(3) }
fn ln<T: L>(t: &T) -> usize { t.len() }
fn ps<T: P>(t: &mut T) { t.push(9); }
fn ts<T: TS>(t: &T) -> String { t.to_string() }
fn uw<T: U>(t: T) -> u8 { t.unwrap() }
fn mn<T: Mn>(a: T, b: T) -> T { a.min(b) }
fn main() {
    println!("{} {}", pw(2u32), 2u32.pow(3));
    let s = String::from("abc");
    println!("{} {}", ln(&s), s.len());
    let mut v: Vec<u8> = Vec::new();
    ps(&mut v);
    println!("{}", v.len());
    println!("{}", ts(&5u8));
    println!("{} {}", uw(Some(1u8)), Some(1u8).unwrap());
    println!("{}", mn(3i32, 4));
}
