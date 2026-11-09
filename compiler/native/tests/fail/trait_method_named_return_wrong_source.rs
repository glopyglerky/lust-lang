// expect: L0734
// rustc: `impl` method signature does not match the trait. The trait ties
// the result to `x`; the impl's elided result borrows from `self`.
trait Pick {
    fn pick<'x>(&self, x: &'x str) -> &'x str;
}

struct S(String);

impl Pick for S {
    fn pick(&self, _x: &str) -> &str {
        &self.0
    }
}

fn main() {
    let s = S(String::from("a"));
    println!("{}", s.pick("b"));
}
