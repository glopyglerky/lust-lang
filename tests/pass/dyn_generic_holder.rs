// ARCH 2.8: dyn references inside a generic struct, built under a `W<&dyn Tr>` expectation.
trait Tr { fn m(&self) -> u32; }
struct A(u32);
impl Tr for A { fn m(&self) -> u32 { self.0 } }
struct W<T> { t: T, k: u32 }
fn total(ws: &Vec<W<&dyn Tr>>) -> u32 { let mut s = 0; for w in ws { s += w.t.m() * w.k; } s }
fn main() {
    let a = A(2);
    let b = A(5);
    let w = W { t: &a as &dyn Tr, k: 3 };
    let ws: Vec<W<&dyn Tr>> = vec![w, W { t: &b, k: 1 }];
    println!("{} {}", ws[0].t.m(), total(&ws));
}
