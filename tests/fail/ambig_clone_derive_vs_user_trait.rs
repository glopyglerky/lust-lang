// expect: L0720
// rustc E0034: the derived `Clone` and a user trait `clone` both apply,
// even though the derived side has no impl method fn to call.
trait Weird {
    fn clone(&self) -> Self;
}

#[derive(Clone, Debug)]
struct M(u8);

impl Weird for M {
    fn clone(&self) -> M {
        M(99)
    }
}

fn main() {
    println!("{:?}", M(1).clone());
}
