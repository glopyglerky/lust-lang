// Drop order and count across dedup, resize (grow, shrink, and
// the unused value), extend_from_slice clones, and split_off.
#[derive(Debug, Clone, PartialEq)]
struct G {
    n: i32,
    tag: String,
}
impl Drop for G {
    fn drop(&mut self) {
        println!("drop {} {}", self.n, self.tag);
    }
}
fn g(n: i32) -> G {
    G { n, tag: format!("t{}", n) }
}
fn main() {
    let mut d = vec![g(1), g(1), g(2), g(2), g(1)];
    println!("-- dedup");
    d.dedup();
    println!("len {}", d.len());
    drop(d);
    let mut r: Vec<G> = Vec::new();
    println!("-- resize 3");
    r.resize(3, g(7));
    for x in &r {
        println!("elem {}", x.n);
    }
    println!("-- resize 1");
    r.resize(1, g(8));
    println!("cap {}", r.capacity());
    drop(r);
    let src = vec![g(4), g(5)];
    let mut e: Vec<G> = Vec::new();
    e.extend_from_slice(&src);
    let t = e.split_off(1);
    println!("-- split {} {}", e.len(), t.len());
    drop(e);
    drop(t);
    drop(src);
    let mut a = vec![g(1)];
    let mut b = vec![g(2), g(3)];
    a.append(&mut b);
    println!("-- appended {} {} {}", a.len(), b.len(), b.capacity());
    drop(a);
    drop(b);
    println!("end");
}
