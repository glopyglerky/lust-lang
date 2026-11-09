// A derive drop that a trait default body still needs (Default::default() in zd1_s): a reject (v4),
// then a header edit of a fn that body calls, on top of it (v5): the header delta checked the
// template again under the old derive list and the derive delta took it as standing (patchseq 7).
mod m { pub const R: u32 = 1; }
fn main() { println!("{}", m::R); }

fn zd1_f(x: u32) -> u32 {
    x
}
#[derive(Clone, Copy, Debug)]
struct Zq1(u32);
trait Zd1 {
    fn zd1_m(&self, q: Zq1) -> u32 {
        let c = || zd1_f(q.0);
        let (a, b) = (q, q);
        c() + a.0 + b.0 + format!("{:?}", q).len() as u32
    }
    fn zd1_s() -> u32 {
        let d: Zq1 = Default::default();
        zd1_f(d.0)
    }
}
impl Zd1 for u8 {}
#[allow(dead_code)]
fn zd1_use() -> u32 {
    1u8.zd1_m(Zq1(2)) + <u8 as Zd1>::zd1_s()
}
