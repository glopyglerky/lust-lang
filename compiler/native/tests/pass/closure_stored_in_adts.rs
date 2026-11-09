// Closures stored in struct fields, Option, Vec, and tuples, called
// through the field (ARCH 2.9 stored-closure class).
struct Holder<F: Fn(u8) -> u8> { f: F, n: u8 }
impl<F: Fn(u8) -> u8> Holder<F> { fn run(&self) -> u8 { (self.f)(self.n) } }
struct Counter<F: FnMut() -> u32> { tick: F }
fn main() {
    let base = 10u8;
    let h = Holder { f: |x| x + base, n: 5 };
    let mut total = 0u32;
    let mut c = Counter { tick: || { total += 1; total } };
    (c.tick)();
    let last = (c.tick)();
    let o = Some(|x: u8| x * 2);
    let r = match o { Some(f) => f(4), None => 0 };
    let v = vec![|x: u8| x + 1];
    let w: Vec<u8> = v.iter().map(|f| f(3)).collect();
    let t = (|| 1u8, 2u8);
    println!("{} {} {} {:?} {} {}", h.run(), last, r, w, (t.0)(), total);
}
