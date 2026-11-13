struct Buf { data: Vec<u8> }
impl Buf {
    const CAP: usize = 4;
    const ZERO: [u8; 2] = [0; 2];
    fn new() -> Buf { Buf { data: Vec::with_capacity(Self::CAP) } }
    fn put(&mut self, b: u8) -> usize { self.data.push(b); self.data.len() }
    fn head(&self) -> [u8; 2] { let mut h = Self::ZERO; h[0] = self.data[0]; h }
}
struct Gen<T> { v: Vec<T> }
impl<T: Clone> Gen<T> {
    const N: usize = 2;
    fn first(&self) -> T where u8: Copy { self.v[0].clone() }
}
fn fill(b: &mut Buf) -> usize { b.put(1) + b.put(2) }
fn main() {
    let mut b = Buf::new();
    let n = fill(&mut b);
    let g = Gen { v: vec![String::from("x")] };
    println!("{} {:?} {} {}", n, b.head(), g.first(), Gen::<u8>::N + Buf::CAP);
}
