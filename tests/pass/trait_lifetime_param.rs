// A trait with a lifetime param, implemented for `'static`.
trait Source<'a> {
    fn get(&self) -> &'a str;
}

struct S;

impl Source<'static> for S {
    fn get(&self) -> &'static str {
        "s"
    }
}

fn main() {
    println!("{}", S.get());
}
