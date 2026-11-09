// A trait-only edit (`s: &str` to `s: &'static str`) while the impl keeps
// `&str` (a refinement rustc allows): callers see the trait's signature,
// so the unchanged `main` must recheck (W-BORROW).
trait Sink {
    fn put(&mut self, s: &str) -> usize;
}

struct V {
    n: usize,
}

impl Sink for V {
    fn put(&mut self, s: &str) -> usize {
        self.n += s.len();
        self.n
    }
}

fn main() {
    let mut v = V { n: 0 };
    let t = String::from("tt");
    let k = v.put(&t);
    println!("{k}");
}
