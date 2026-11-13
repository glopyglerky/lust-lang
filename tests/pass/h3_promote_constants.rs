// Rvalue static promotion: `&5`, `&CONST`, `&None`, `&E::A`, `&(1, true)`,
// `&[1, 2, 3]`, `&P { .. }`, and arithmetic over those borrow a `'static`
// value, so they may be returned and kept past their statement.
use std::ops::Deref;
#[derive(Debug, Clone, Copy, PartialEq)]
struct P { x: i32, y: u8 }
#[derive(Debug)]
enum E { A }
const C: i32 = 7;
struct L;
impl Deref for L { type Target = i32; fn deref(&self) -> &i32 { &5 } }
fn a() -> &'static i32 { &C }
fn b() -> &'static P { &P { x: 1, y: 2 } }
fn c() -> &'static Option<i32> { &None }
fn d() -> &'static E { &E::A }
fn e() -> &'static (i64, bool) { &(-3, true) }
fn f() -> &'static [u8; 3] { &[1, 2, 3] }
fn g() -> &'static i32 { &(1 + 2 * 3) }
fn keep<'a, T>(t: &'a T) -> &'a T { t }
fn main() {
    let k = keep(&5i32);
    let r;
    {
        r = &(4, 'z');
    }
    let mut v: Vec<&i32> = Vec::new();
    for _ in 0..3 {
        v.push(&4);
    }
    println!("{} {:?} {:?} {:?} {:?} {:?} {}", a(), b(), c(), d(), e(), f(), g());
    println!("{} {:?} {:?} {}", k, r, v, *L + 1);
}
