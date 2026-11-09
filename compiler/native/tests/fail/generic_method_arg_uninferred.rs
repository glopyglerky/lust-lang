// expect: L0610
// rustc E0283: the method's own parameter `T` is never fixed.
struct S;

impl S {
    fn g<T: Default>(&self) -> usize {
        3
    }
}

fn main() {
    println!("{}", S.g());
}
