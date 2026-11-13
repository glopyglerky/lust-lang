// Moving out of a Box, whole and by field; drop order after.
struct G { n: i32, s: String }
impl Drop for G { fn drop(&mut self) { println!("drop G{}", self.n); } }
struct H { n: i32, s: String }
fn main() {
    let b = Box::new(G { n: 1, s: "a".to_string() });
    let g = *b;
    println!("moved {} {}", g.n, g.s);
    let b2 = Box::new(H { n: 2, s: "b".to_string() });
    let s2 = b2.s;
    println!("field {}", s2);
    println!("rest {}", b2.n);
    let opts = Some(Box::new(G { n: 3, s: "c".to_string() }));
    match opts { Some(inner) => { let g3 = *inner; println!("opt {}", g3.n); } None => {} }
    println!("end");
}
