// expect: L0110
// rustc E0308: in `vec![..]`, a fn with its own late-bound lifetime does
// not join a method whose impl lifetime is early-bound (W-BORROW3).
struct P<'p> { r: &'p u8, n: u8 }
impl<'p> P<'p> {
    fn m(&mut self) -> u8 { self.n += 1; *self.r + self.n }
    fn go(&mut self) -> u8 {
        let fs: Vec<fn(&mut P<'p>) -> u8> = vec![P::m, pe];
        fs.iter().map(|f| f(self)).sum()
    }
}
fn pe(p: &mut P<'_>) -> u8 { p.n }
static G: u8 = 7;
fn main() { let mut p = P { r: &G, n: 0 }; println!("{}", p.go()); }
