// expect: L0110
// rustc E0277: a default body taking `self` by value needs
// `Self: Sized`, which a trait's Self does not have by default.
trait T {
    fn f(self) -> u8 {
        1
    }
}

impl T for u8 {}

fn main() {
    println!("{}", 3u8.f());
}
