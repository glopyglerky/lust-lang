// expect: L0734
// rustc: the impl demands a 'static argument where the trait takes any.
trait Take {
    fn take(&self, x: &str) -> usize;
}

struct S;

impl Take for S {
    fn take(&self, x: &'static str) -> usize {
        x.len()
    }
}

fn main() {
    println!("{}", S.take("a"));
}
