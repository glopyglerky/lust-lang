// Outlives bounds: `'a: 'static` on a fn, and struct and enum facts that
// signatures naming the type may assume (implied bounds).
struct H<'a, 'b: 'a> { r: &'a u8, s: &'b u8 }
fn longer<'a, 'b>(h: H<'a, 'b>) -> &'a u8 { h.s }
fn main1() { let v = 3u8; let w = 4u8; let h = H { r: &v, s: &w }; println!("{} {}", longer(h), v); }
enum E<'a, 'b: 'a> { Both(&'a u8, &'b u8), One(&'a u8) }
fn pick<'a, 'b>(e: E<'a, 'b>) -> &'a u8 { match e { E::Both(_, s) => s, E::One(r) => r } }
fn main2() { let v = 1u8; let w = 2u8; println!("{} {}", pick(E::Both(&v, &w)), pick(E::One(&v))); }
fn f<'a: 'static>(x: &'a u8) -> &'static u8 { x }
static V: u8 = 3;
fn main3() { println!("{}", f(&V)); }
fn main() { main1(); main2(); main3(); }
