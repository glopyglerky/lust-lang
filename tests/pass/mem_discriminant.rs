// std::mem::discriminant and Discriminant<T>: the value, Eq, Hash (the
// repr type's write call), and Debug ("Discriminant(N)") match rustc.
// ts-rust's generated NodeData::same compares two discriminants.
use std::hash::{Hash, Hasher};
use std::mem::{self, discriminant, Discriminant};
use std::collections::HashMap;
struct Log(Vec<String>);
impl Hasher for Log {
    fn finish(&self) -> u64 { 0 }
    fn write(&mut self, b: &[u8]) { self.0.push(format!("w{:?}", b)); }
    fn write_u8(&mut self, v: u8) { self.0.push(format!("u8 {v}")); }
    fn write_i8(&mut self, v: i8) { self.0.push(format!("i8 {v}")); }
    fn write_i16(&mut self, v: i16) { self.0.push(format!("i16 {v}")); }
    fn write_u32(&mut self, v: u32) { self.0.push(format!("u32 {v}")); }
    fn write_i64(&mut self, v: i64) { self.0.push(format!("i64 {v}")); }
    fn write_isize(&mut self, v: isize) { self.0.push(format!("isize {v}")); }
    fn write_usize(&mut self, v: usize) { self.0.push(format!("usize {v}")); }
    fn write_u128(&mut self, v: u128) { self.0.push(format!("u128 {v}")); }
}
fn h<T: Hash>(t: &T) -> String { let mut l = Log(Vec::new()); t.hash(&mut l); l.0.join(",") }
#[derive(Clone, Copy)] enum Plain { A, B(u8), C { x: i32 } }
#[repr(u8)] #[derive(Clone, Copy)] enum U8e<'a> { A(&'a str), B = 7, C(u32) }
#[repr(i8)] enum I8e { A = -3, B, C = 100 }
#[repr(C)] enum Ce { A = 5, B }
#[repr(isize)] enum Ie { A = -9 }
#[repr(i64)] enum I64e { A = -9 }
#[repr(usize)] enum Ue { A = 9 }
#[repr(u128)] enum Big { A = 1, B = 340282366920938463463374607431768211455 }
#[repr(u32)] enum Gap { A = 10, B = 4000000000, C }
enum One { Only(String) }
struct S(u8);
fn same<T>(a: &T, b: &T) -> bool { mem::discriminant(a) == mem::discriminant(b) }
struct Holder { d: Discriminant<Plain> }
#[allow(enum_intrinsics_non_enums)]
fn nonenum() {
    println!("{:?} {}", discriminant(&S(4)), h(&discriminant(&S(4))));
    println!("{:?} {}", discriminant(&5u32), same(&Plain::A, &Plain::A));
    println!("{:?}", discriminant(&&Plain::A));
}
fn main() {
    let ds = [discriminant(&Plain::A), discriminant(&Plain::B(1)), discriminant(&Plain::C { x: 2 })];
    for d in &ds { println!("{:?} {:#?} {:x?} {}", d, d, d, h(d)); }
    let u = [U8e::A("x"), U8e::B, U8e::C(3)];
    for v in &u { let d = discriminant(v); println!("{:?} {}", d, h(&d)); }
    for v in [I8e::A, I8e::B, I8e::C] { let d = discriminant(&v); println!("{:?} {:x?} {}", d, d, h(&d)); }
    for v in [Ce::A, Ce::B] { let d = discriminant(&v); println!("{:?} {}", d, h(&d)); }
    println!("{:?} {} {:?} {} {:?} {}", discriminant(&Ie::A), h(&discriminant(&Ie::A)), discriminant(&I64e::A), h(&discriminant(&I64e::A)), discriminant(&Ue::A), h(&discriminant(&Ue::A)));
    for v in [Big::A, Big::B] { let d = discriminant(&v); println!("{:?} {}", d, h(&d)); }
    for v in [Gap::A, Gap::B, Gap::C] { let d = discriminant(&v); println!("{:?} {}", d, h(&d)); }
    println!("{:?} {}", discriminant(&One::Only(String::new())), h(&discriminant(&One::Only(String::new()))));
    let o: Option<u8> = Some(3); let r: Result<u8, ()> = Err(());
    println!("{:?} {:?} {} {}", discriminant(&o), discriminant(&r), h(&discriminant(&o)), h(&discriminant(&None::<String>)));
    nonenum();
    println!("{} {}", same(&U8e::B, &U8e::C(1)), same(&Some(1), &Some(2)));
    let mut m: HashMap<Discriminant<Plain>, u32> = HashMap::new();
    for v in [Plain::A, Plain::B(2), Plain::A, Plain::C { x: 1 }, Plain::B(9)] { *m.entry(discriminant(&v)).or_insert(0) += 1; }
    let mut c: Vec<_> = m.iter().map(|(k, v)| format!("{k:?}={v}")).collect(); c.sort(); println!("{c:?}");
    let hd = Holder { d: discriminant(&Plain::B(0)) }; let hd2 = hd.d; println!("{:?} {}", hd2, hd2 != ds[0]);
    println!("{:?}", ds.iter().filter(|d| **d == ds[1]).count());
}
