// `-x` on an integer variable that ends signed, and `to_string` through
// written Display impls (plain, by reference, and generic).
use std::fmt;
struct Yes(i32);
impl fmt::Display for Yes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Y{}", self.0)
    }
}
struct G<T>(T);
impl<T: fmt::Display> fmt::Display for G<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "G{}", self.0)
    }
}
fn g<T>(t: T) -> T {
    t
}
fn main() {
    let a = -5;
    let b: i64 = a;
    let c = -(3 + 4);
    let d = g(-2);
    let f = -1;
    let v: Vec<i16> = vec![f];
    println!("{} {} {} {} {:?}", a, b, c, d, v);
    let y = Yes(3);
    println!("{} {} {} {}", y.to_string(), (&y).to_string(), G(5).to_string(), G(Yes(1)).to_string());
}
