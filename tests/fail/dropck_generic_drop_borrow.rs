// expect: L0306
// rustc E0597: `W`'s Drop may read the borrow, so `x` must outlive `w` (ARCH 5.4).
struct W<T: std::fmt::Debug>(T);

impl<T: std::fmt::Debug> Drop for W<T> {
    fn drop(&mut self) {
        println!("{:?}", self.0);
    }
}

fn main() {
    let w;
    let x = String::from("hi");
    w = W(&x);
}
