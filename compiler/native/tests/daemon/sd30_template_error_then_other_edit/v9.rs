// A derive drop that a generic fn's body needs (q == q), the use removed and restored (v8: a 
// template error the kept program answers), then a plain body edit (v9): the template's error must 
// stand.
// gen_dfrm
mod m { pub const R: u32 = 1; }
#[derive(Clone, Copy, Debug, Default, Hash)]
struct Zq(u32);
fn helper(x: u32) -> u32 {
    x
}
fn plain_use(q: Zq) -> u32 {
    let mut n = helper(1) as u32;
    let d: Zq = Default::default(); n += d.0;
    n += 5;
    n
}
fn gen_use<T: Copy>(t: T, q: Zq) -> (T, u32) {
    let mut n = 0u32;
    n += (q == q) as u32;
    (t, n)
}
trait Tr {
    fn tdef(&self, q: Zq) -> u32 {
        let mut n = helper(2) as u32;
        
        n
    }
    fn tassoc() -> u32 {
        let q = Zq(3);
        let mut n = 0u32;
        let (a, b) = (q, q); n += a.0 + b.0;
        n
    }
}
impl Tr for u8 {}
fn clo_use(q: Zq) -> u32 {
    let f = move || {
        let mut n = 0u32;
        
        n
    };
    f()
}
impl Zq {
    fn meth(&self, q: Zq) -> u32 {
        let mut n = self.0;
        n += q.clone().0;
        n
    }
}
fn main() {
    let q = Zq(4);
    println!("{} {} {} {} {} {}", plain_use(Zq(1)), gen_use(1u8, Zq(2)).1, 1u8.tdef(Zq(5)), <u8 as Tr>::tassoc(), clo_use(Zq(6)), q.meth(Zq(7)) + m::R);
}
