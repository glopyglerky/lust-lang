use std::sync::Arc;
#[derive(Debug, Clone)]
struct S { n: u32, name: String }
fn main() {
    let v = vec![S { n: 1, name: "a".into() }, S { n: 2, name: "b".into() }];
    let a: Arc<[S]> = v.into();
    let b = a.clone();
    for s in b.iter() { println!("{} {}", s.n, s.name); }
    println!("{:?}", a);
    let e: Arc<[u8]> = Arc::from(&b"xyz"[..]);
    println!("{:?} {}", e, e.len());
}
