// ARCH 2.8: `&mut dyn` passed on and reborrowed; arrays of `&mut dyn`.
trait Acc { fn add(&mut self, x: i64); fn total(&self) -> i64; }
struct Sum { t: i64 }
struct Max { m: i64 }
impl Acc for Sum { fn add(&mut self, x: i64) { self.t += x; } fn total(&self) -> i64 { self.t } }
impl Acc for Max { fn add(&mut self, x: i64) { if x > self.m { self.m = x; } } fn total(&self) -> i64 { self.m } }
fn feed(a: &mut dyn Acc, xs: &[i64]) { for x in xs { a.add(*x); } }
fn report(a: &dyn Acc) -> i64 { a.total() }
fn main() {
    let mut s = Sum { t: 0 };
    let mut m = Max { m: -100 };
    let data = [3i64, -1, 7, 2];
    feed(&mut s, &data);
    feed(&mut m, &data);
    let r1 = report(&s);
    s.t += 1;
    {
        let d: &mut dyn Acc = &mut s;
        d.add(10);
        feed(d, &data);
    }
    println!("{} {} {}", r1, report(&s), report(&m));
    let accs: [&mut dyn Acc; 2] = [&mut s, &mut m];
    let mut total = 0;
    let mut accs = accs; let mut i = 0; while i < 2 { accs[i].add(1); total += accs[i].total(); i += 1; }
    println!("{}", total);
}
