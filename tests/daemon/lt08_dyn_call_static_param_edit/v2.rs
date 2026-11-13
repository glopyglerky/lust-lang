// A call through `&mut dyn Sink` takes the trait method's signature: the
// edit to `&'static str` (trait and impl) makes the local borrow too
// short, and the unchanged `main` must recheck (W-BORROW).
trait Sink {
    fn put(&mut self, s: &'static str) -> usize;
}

struct V {
    n: usize,
}

impl Sink for V {
    fn put(&mut self, s: &'static str) -> usize {
        self.n += s.len();
        self.n
    }
}

fn main() {
    let mut v = V { n: 0 };
    let t = String::from("tt");
    let d: &mut dyn Sink = &mut v;
    let k = d.put(&t);
    println!("{k}");
}
