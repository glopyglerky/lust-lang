// ARCH 2.8: a visitor: `&mut dyn Visitor` and `&dyn Node` call each other.
trait Node { fn accept(&self, v: &mut dyn Visitor); fn val(&self) -> i64; }
trait Visitor { fn visit(&mut self, n: &dyn Node); }
struct Leaf(i64);
struct Pair { l: Leaf, r: Leaf }
impl Node for Leaf { fn accept(&self, v: &mut dyn Visitor) { v.visit(self); } fn val(&self) -> i64 { self.0 } }
impl Node for Pair { fn accept(&self, v: &mut dyn Visitor) { self.l.accept(v); self.r.accept(v); v.visit(self); } fn val(&self) -> i64 { self.l.0 * self.r.0 } }
struct Sum { total: i64, count: u32 }
impl Visitor for Sum { fn visit(&mut self, n: &dyn Node) { self.total += n.val(); self.count += 1; } }
fn main() {
    let p = Pair { l: Leaf(3), r: Leaf(4) };
    let mut s = Sum { total: 0, count: 0 };
    p.accept(&mut s);
    println!("{} {}", s.total, s.count);
    let n: &dyn Node = &p;
    n.accept(&mut s);
    println!("{} {}", s.total, s.count);
}
