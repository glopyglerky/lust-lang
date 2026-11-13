// expect: L0720
// rustc E0034: `PartialOrd::lt` and a user trait `lt` both apply.
trait Weird {
    fn lt(&self, other: &Self) -> bool;
}

impl Weird for u8 {
    fn lt(&self, _other: &u8) -> bool {
        true
    }
}

fn main() {
    println!("{}", 1u8.lt(&2u8));
}
