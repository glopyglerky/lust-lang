// Generic block impls: an inherent impl and a std trait impl of a block generic type.
use std::fmt::Display;
fn main() {
    struct W<T>(T);
    impl<T: Display> Display for W<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "[{}]", self.0) }
    }
    impl<T: Clone> W<T> { fn dup(&self) -> (T, T) { (self.0.clone(), self.0.clone()) } }
    println!("{} {}", W(1), W("a"));
    println!("{:?}", W(5u8).dup());
}
