// Every `&mut` argument that coercion reborrows, with a later argument,
// is two-phase (rustc): `g(a, b, a.len() + b.len())`, a two-phase method
// receiver with a reborrowed argument, and reservations between reads.
fn g(a: &mut Vec<u32>, b: &mut Vec<u32>, n: usize) -> usize { a.push(n as u32); b.push(1); a.len() * 10 + b.len() }
fn gm(a: &mut Vec<u32>, n: usize, b: &mut Vec<u32>, m: usize) -> usize { a.push(n as u32); b.push(m as u32); a.len() * 10 + b.len() }
struct H { v: Vec<u32> }
impl H { fn g2(&mut self, b: &mut Vec<u32>, n: usize) -> usize { self.v.push(n as u32); b.push(1); self.v.len() + b.len() } }
fn gen_f<F: FnMut(&mut Vec<u32>, &mut Vec<u32>, usize) -> usize>(mut f: F, a: &mut Vec<u32>, b: &mut Vec<u32>) -> usize { f(a, b, a.len() + b.len()) }
fn main() {
    let mut x = vec![1u32];
    let mut y = vec![2u32, 3];
    let a = &mut x;
    let b = &mut y;
    println!("{}", g(a, b, a.len() + b.len()));
    println!("{}", gm(a, a.len(), b, b.len() + a.len()));
    let mut h = H { v: vec![] };
    println!("{}", h.g2(b, h.v.len() + b.len()));
    println!("{}", gen_f(|p, q, n| { p.push(n as u32); p.len() + q.len() }, a, b));
    println!("{:?} {:?}", x, y);
}
