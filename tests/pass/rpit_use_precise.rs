// `use<..>` precise capturing (edition 2024): the opaque value captures
// only the listed params, so a borrow passed for another param ends
// at the call. Drop order output shows the hidden `Loud` drops last.
struct C {
    v: Vec<u32>,
    n: u8,
}
struct Loud(u32);
impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn first<'a>(c: &'a C, _x: &u8) -> impl Iterator<Item = &'a u32> + use<'a> {
    c.v.iter()
}
fn none(c: &C) -> impl Sized + use<> {
    Loud(c.v[0])
}
fn gen_iter<'a, T>(v: &'a [T], _x: &u8) -> impl Iterator<Item = &'a T> + use<'a, T> {
    v.iter()
}
struct M<'s> {
    s: &'s [u32],
}
impl<'s> M<'s> {
    fn items(&self) -> impl Iterator<Item = &'s u32> + use<'s> {
        self.s.iter()
    }
}
fn elided(c: &C) -> impl Iterator<Item = &u32> + use<'_> {
    c.v.iter().rev()
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 7 };
    let mut b = 5u8;
    let it = first(&c, &b);
    b += 1;
    let s: u32 = it.sum();
    let g = gen_iter(&c.v, &b);
    b += 1;
    println!("{} {}", s, g.count());
    let it = M { s: &c.v }.items();
    println!("{}", it.count());
    let e: Vec<&u32> = elided(&c).collect();
    println!("{:?}", e);
    let l = none(&c);
    c.n = 9;
    println!("{} {}", c.n, b);
}
