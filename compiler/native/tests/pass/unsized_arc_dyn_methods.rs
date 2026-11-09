use std::sync::Arc;
trait Shape { fn area(&self) -> f64; fn name(&self) -> String { String::from("shape") } }
struct Sq(f64); struct Ci(f64);
impl Shape for Sq { fn area(&self) -> f64 { self.0 * self.0 } fn name(&self) -> String { "sq".into() } }
impl Shape for Ci { fn area(&self) -> f64 { 3.0 * self.0 * self.0 } }
fn total(v: &[Arc<dyn Shape>]) -> f64 { v.iter().map(|s| s.area()).sum() }
fn main() {
    let v: Vec<Arc<dyn Shape>> = vec![Arc::new(Sq(2.0)), Arc::new(Ci(1.0))];
    let c = v.clone();
    println!("{} {} {}", total(&v), c[0].name(), c[1].name());
    println!("{} {}", Arc::strong_count(&v[0]), Arc::ptr_eq(&v[0], &c[0]));
}
