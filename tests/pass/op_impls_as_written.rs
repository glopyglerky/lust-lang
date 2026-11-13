// Operator impls apply to the operand types as written: impls for
// references (`impl Add<&V> for &V`, `impl Neg for &V`), a reference right
// side for an assignment operator, and a scalar left side with a user
// type on the right (`impl Mul<V> for i32`).
use std::ops::{Add, AddAssign, Mul, Neg};
#[derive(Debug)]
struct L(Vec<i32>);
impl Add<&L> for &L { type Output = L; fn add(self, o: &L) -> L { L(vec![self.0[0] + o.0[0]]) } }
#[derive(Debug, Clone, Copy)]
struct V(i32);
impl AddAssign<&V> for V { fn add_assign(&mut self, o: &V) { self.0 += o.0; } }
impl Neg for &V { type Output = V; fn neg(self) -> V { V(-self.0) } }
impl Mul<V> for i32 { type Output = V; fn mul(self, v: V) -> V { V(self * v.0) } }
impl Mul<V> for f64 { type Output = f64; fn mul(self, v: V) -> f64 { self * v.0 as f64 } }
fn main() {
    let a = L(vec![1, 2]);
    let b = L(vec![10, 20]);
    let c = &a + &b;
    println!("{:?} {:?}", c, a);
    let mut x = V(1);
    let y = V(2);
    x += &y;
    println!("{:?} {:?}", x, -&y);
    println!("{:?} {:?} {}", 3 * V(4), 2i32 * V(5), 0.5 * V(3));
}
