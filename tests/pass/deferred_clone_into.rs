// Method calls on a value whose type a later statement fixes: the clone of
// an open std container, and into() to a later target, as rustc's pending
// obligations.
#[derive(Debug)]
struct Lc(u8);
impl Clone for Lc {
    fn clone(&self) -> Lc {
        println!("clone Lc({})", self.0);
        Lc(self.0 + 1)
    }
}
fn take_vec(v: Vec<u8>) -> usize {
    v.len()
}
fn main() {
    let mut v = Vec::new();
    let w = v.clone();
    v.push(Lc(1));
    let w2 = v.clone();
    println!("{v:?} {w:?} {w2:?}");
    let mut o = None;
    let p = o.clone();
    o = Some(String::from("s"));
    println!("{o:?} {p:?}");
    let mut m = std::collections::HashMap::new();
    let c = m.clone();
    m.insert(1u8, Lc(5));
    println!("{} {}", m.len(), c.len());
    let s: &[u8] = &[1, 2, 3];
    let x = s.into();
    println!("{}", take_vec(x));
    let a = 5u8.into();
    let b: u64 = a;
    let t = "hi".into();
    let u: String = t;
    let bs = String::from("bx").into();
    let bb: Box<str> = bs;
    let rc = "rc".into();
    let rr: std::rc::Rc<str> = rc;
    println!("{b} {u} {bb} {rr}");
}
