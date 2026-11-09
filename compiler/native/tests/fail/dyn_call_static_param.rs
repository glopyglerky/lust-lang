// expect: L0306
// rustc E0597: a call through `&mut dyn Sink` takes the trait method's
// signature, whose `&'static str` parameter a local borrow cannot meet.
trait Sink { fn put(&mut self, s: &'static str); }
struct V { v: Vec<&'static str> }
impl Sink for V { fn put(&mut self, s: &'static str) { self.v.push(s); } }
fn main() {
    let mut v = V { v: Vec::new() };
    let t = String::from("t");
    let d: &mut dyn Sink = &mut v;
    d.put(&t);
    println!("{}", v.v.len());
}
