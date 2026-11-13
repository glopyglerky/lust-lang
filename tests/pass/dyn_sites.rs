// ARCH 2.8: coercion sites: let, if/match arms, blocks, vec!, casts, args, returns, assignment, temporaries.
trait Tr { fn v(&self) -> u32; }
struct A; struct B(u32);
impl Tr for A { fn v(&self) -> u32 { 1 } }
impl Tr for B { fn v(&self) -> u32 { self.0 } }
struct Holder { t: &'static dyn Tr }
fn ret_dyn(b: &B) -> &dyn Tr { b }
fn main() {
    let a = A; let b = B(7);
    let c = true;
    let x: &dyn Tr = if c { &a } else { &b };
    let y = match b.0 { 7 => &b as &dyn Tr, _ => &a };
    let z: &dyn Tr = { &b };
    let v: Vec<&dyn Tr> = vec![&a, &b, &a];
    let mut sum = 0;
    for t in &v { sum += t.v(); }
    let w = &a as &dyn Tr;
    println!("{} {} {} {} {} {}", x.v(), y.v(), z.v(), sum, w.v(), ret_dyn(&b).v());
    let mut m: &dyn Tr = &a;
    println!("{}", m.v());
    m = &b;
    println!("{}", m.v());
    let tmp: &dyn Tr = &B(40);
    println!("{}", tmp.v());
}
