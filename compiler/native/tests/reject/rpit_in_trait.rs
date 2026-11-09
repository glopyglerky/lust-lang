// expect: L0602
// `impl Trait` in a trait method's return type is not supported yet.
trait Source {
    fn items(&self) -> impl Iterator<Item = u8>;
}

struct S;

impl Source for S {
    fn items(&self) -> impl Iterator<Item = u8> {
        vec![1u8].into_iter()
    }
}

fn main() {
    println!("{}", S.items().count());
}
