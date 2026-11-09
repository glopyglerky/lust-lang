use std::sync::Arc;
fn total(v: &[Arc<i64>]) -> i64 { v.iter().map(|x| **x).sum() }
fn main() {
    let a = Arc::new(Arc::new(4));
    let b = Arc::clone(&*a);
    println!("{} {} {}", a, Arc::strong_count(&b), **a + 1);
    let v: Vec<Arc<i64>> = (1..4).map(|i| Arc::new(i)).collect();
    println!("{} {:?}", total(&v), v);
    let o: Option<Arc<u8>> = None;
    println!("{} {}", std::mem::size_of::<Option<Arc<u8>>>(), o.is_none());
    let e = Arc::new(5) == Arc::new(5);
    let f = Arc::new("x") < Arc::new("y");
    println!("{} {}", e, f);
}
