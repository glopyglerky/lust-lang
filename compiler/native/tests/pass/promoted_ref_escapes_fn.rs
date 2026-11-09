// Promoted constants and const reference pointees have static storage:
// a reference to one may escape its function (a deep call then reuses
// the stack it would have used).
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
const S: &D = &D(7);
const T: (u8, &D) = (1, &D(8));
const O: Option<&D> = Some(&D(9));
const RR: &&D = &&D(10);
struct W { r: &'static D }
const WW: W = W { r: &D(11) };
const STR: &[&str] = &["a", "b"];
trait Tr { const A: &'static D; }
struct K;
impl Tr for K { const A: &'static D = &D(12); }
fn g1() -> &'static D { T.1 }
fn g2() -> &'static D { O.unwrap() }
fn g3() -> &'static D { *RR }
fn g4() -> &'static D { WW.r }
fn g5<X: Tr>() -> &'static D { X::A }
fn g6() -> &'static D { const { &D(13) } }
fn g7(n: u32) -> &'static D { if n == 0 { S } else { let r = g7(n - 1); r } }
fn g8() -> &'static [&'static str] { STR }
fn g9() -> &'static i32 { &(1 + 2) }
fn g10() -> &'static [u8] { &[1, 2, 3] }
fn g11() -> &'static (u8, char) { &(3, 'x') }
fn smash(n: u64) -> u64 { let a = [n; 256]; a.iter().sum() }
fn main() {
    let c = || -> &'static D { S };
    let rs = [g1(), g2(), g3(), g4(), g5::<K>(), g6(), g7(5), c()];
    let a = g9(); let b = g10(); let e = g11(); let s = g8();
    println!("{}", smash(3));
    for r in rs { println!("{}", r.0); }
    println!("{} {:?} {:?} {:?}", a, b, e, s);
}
