// An elided parameter through dyn borrows only for the call.
trait Sink { fn put(&mut self, s: &str) -> usize; }
struct V { n: usize }
impl Sink for V { fn put(&mut self, s: &str) -> usize { self.n += s.len(); self.n } }
fn main() {
    let mut v = V { n: 0 };
    let k;
    {
        let t = String::from("tt");
        let d: &mut dyn Sink = &mut v;
        k = d.put(&t);
    }
    println!("{} {}", k, v.n);
}
