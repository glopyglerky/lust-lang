// Calls through `T: Tr` take the trait's regions: a `&'static str`
// parameter passes through, and an elided result borrows the receiver.
trait Tr { fn put(&mut self, s: &'static str) -> usize; fn name(&self) -> &str; }
struct V { v: Vec<&'static str>, n: String }
impl Tr for V {
    fn put(&mut self, s: &'static str) -> usize { self.v.push(s); self.v.len() }
    fn name(&self) -> &str { &self.n }
}
fn go<T: Tr>(t: &mut T, s: &'static str) -> usize { t.put(s) }
fn nm<T: Tr>(t: &T) -> &str { t.name() }
fn main() {
    let mut v = V { v: Vec::new(), n: String::from("v") };
    let k = go(&mut v, "x");
    println!("{} {}", k, nm(&v));
}
