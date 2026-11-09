// `mem::replace` and `mem::take` return what `*dest` held: the result
// keeps the old value's borrows, not the `&mut` borrow of `dest`, so
// `self.v = out` after `replace(&mut self.v, ..)` is fine (rustc).
struct M<K> {
    e: Vec<K>,
}
impl<K: PartialOrd> M<K> {
    fn app(&mut self, other: &mut M<K>) {
        let left = std::mem::replace(&mut self.e, Vec::new());
        let right = std::mem::take(&mut other.e);
        let mut out = Vec::new();
        for x in left {
            out.push(x);
        }
        for x in right {
            out.push(x);
        }
        self.e = out;
    }
}
struct S<'a> {
    v: Vec<&'a str>,
    n: Option<&'a str>,
}
impl<'a> S<'a> {
    fn f(&mut self) -> Option<&'a str> {
        let o = std::mem::take(&mut self.n);
        self.n = self.v.pop();
        o
    }
}
fn main() {
    let mut a = M { e: vec!["a", "c"] };
    let mut b = M { e: vec!["b"] };
    a.app(&mut b);
    println!("{:?} {}", a.e, b.e.len());
    let s = String::from("a");
    let r;
    {
        let mut v = vec![s.as_str()];
        r = std::mem::replace(&mut v, vec![]);
        v.push("z");
        println!("{:?}", v);
    }
    println!("{:?}", r);
    let mut t = S { v: vec!["x", "y"], n: Some("z") };
    let a = t.f();
    let b = t.f();
    println!("{:?} {:?} {:?}", a, b, t.n);
}
