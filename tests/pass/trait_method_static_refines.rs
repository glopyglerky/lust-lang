// rustc accepts: an impl may return a 'static reference where the trait
// borrows from self, and may take any lifetime where the trait wants 'static.
trait Name {
    fn name(&self) -> &str;
    fn keep(&self, x: &'static str) -> usize;
}

struct S;

impl Name for S {
    fn name(&self) -> &'static str {
        "s"
    }
    fn keep(&self, x: &str) -> usize {
        x.len()
    }
}

fn main() {
    println!("{} {}", S.name(), S.keep("ab"));
}
