// An argument that coercion reborrows (`self` as `&mut C`) is two-phase
// in any position and for overloaded calls too (R12).
struct C { symbols: Vec<u32>, n: u32 }
fn id(s: &Vec<u32>) -> u32 { s.len() as u32 }
fn f3(_k: u32, c: &mut C, j: u32) -> u32 { c.n += j; c.n }
impl C {
    fn a(&mut self) -> u32 { f3(1, self, self.n) }
    fn go(&mut self, callback: &mut dyn FnMut(&mut C, u32) -> bool) -> bool { callback(self, id(&self.symbols)) }
    fn go2<F: FnMut(&mut C, u32) -> bool>(&mut self, mut cb: F) -> bool { cb(self, id(&self.symbols)) }
    fn go3(&mut self, cb: &dyn Fn(u32, &mut C, u32) -> u32) -> u32 { cb(1, self, self.n) }
    fn go4(&mut self) -> u32 { let mut cl = |c: &mut C, k: u32| { c.n += k; c.n }; cl(self, id(&self.symbols)) }
}
fn main() {
    let mut c = C { symbols: vec![1], n: 0 };
    let r = c.go(&mut |c, k| { c.n += k; true });
    let r2 = c.go2(|c, k| { c.n += k; true });
    let r3 = c.go3(&|a, c, k| { c.n += a + k; c.n });
    println!("{} {} {} {} {} {}", r, r2, r3, c.a(), c.go4(), c.n);
}
