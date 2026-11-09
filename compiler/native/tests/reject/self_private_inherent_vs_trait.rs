// expect: L0510
// rustc skips an inaccessible inherent associated fn and takes the
// trait's (prints 5). lust rejects instead; before the `Self::` privacy
// check it called the private inherent fn and printed 1.
mod pa {
    pub struct S;
    impl S {
        fn t() -> i32 {
            1
        }
    }
}
trait Tr {
    fn t() -> i32;
}
impl Tr for pa::S {
    fn t() -> i32 {
        5
    }
}
impl pa::S {
    fn u() -> i32 {
        Self::t()
    }
}
fn main() {
    println!("{}", pa::S::u());
}
