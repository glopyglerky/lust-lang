// rustc accepts: impls that fit the trait's lifetimes (the same tie, a
// longer-lived result, or a named spelling of the elided one).
trait Pick {
    fn pick<'x>(&self, x: &'x str) -> &'x str;
    fn name(&self) -> &str;
}

struct S(String);

impl Pick for S {
    fn pick<'y>(&self, x: &'y str) -> &'y str {
        x
    }
    fn name<'s>(&'s self) -> &'s str {
        &self.0
    }
}

fn main() {
    let s = S(String::from("a"));
    println!("{} {}", s.pick("b"), s.name());
}
