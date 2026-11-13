// Comparison operators and default methods through bounds: the template
// checks through the bound, each instance calls the written impl (or the
// core and derived comparison) with rustc's call order.
use std::cmp::Ordering;
#[derive(Debug, Clone)]
struct X(u8);
impl PartialEq for X { fn eq(&self, o: &X) -> bool { println!("eq"); self.0 == o.0 } }
impl Eq for X {}
impl PartialOrd for X { fn partial_cmp(&self, o: &X) -> Option<Ordering> { println!("pcmp"); Some(self.cmp(o)) } }
impl Ord for X { fn cmp(&self, o: &X) -> Ordering { println!("cmp"); o.0.cmp(&self.0) } }
fn lt<T: PartialOrd>(a: &T, b: &T) -> bool { a < b }
fn ne<T: PartialEq>(a: &T, b: &T) -> bool { a != b && a.ne(b) }
fn mx<T: Ord>(a: T, b: T) -> T { a.max(b) }
fn cl<T: Ord>(a: T, lo: T, hi: T) -> T { a.clamp(lo, hi) }
fn le_m<T: PartialOrd>(a: &T, b: &T) -> bool { a.le(b) }
fn main() {
  println!("{} {} {}", lt(&X(1), &X(2)), lt(&1, &2), lt(&"a".to_string(), &"b".to_string()));
  println!("{} {}", ne(&X(1), &X(1)), ne(&3u8, &4u8));
  println!("{:?} {} {:?}", mx(X(1), X(2)), mx(3, 9), mx(String::from("b"), String::from("a")));
  println!("{:?} {} {:?}", cl(X(5), X(9), X(1)), cl(5, 1, 3), cl('q', 'a', 'f'));
  println!("{} {}", le_m(&X(2), &X(1)), le_m(&1.5, &f64::NAN));
}
