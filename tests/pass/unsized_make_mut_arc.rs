use std::sync::Arc;
fn main() {
    let mut a: Arc<[i32]> = Arc::from(vec![1, 2, 3]);
    let b = a.clone();
    Arc::make_mut(&mut a)[0] = 10;
    println!("{:?} {:?} {}", a, b, Arc::strong_count(&b));
    if let Some(m) = Arc::get_mut(&mut a) { m[1] = 20; }
    println!("{:?}", a);
    let mut d: Arc<str> = Default::default();
    Arc::make_mut(&mut d);
    println!("{:?} {}", d, Arc::strong_count(&d));
    let mut s: Arc<[String]> = Arc::from(vec![String::from("p")]);
    let s2 = s.clone();
    Arc::make_mut(&mut s)[0].push('q');
    println!("{:?} {:?}", s, s2);
}
