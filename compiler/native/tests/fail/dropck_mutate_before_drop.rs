// expect: L0303
// rustc E0502: `w` is dropped at the end of `main`, and its Drop reads `&x`.
struct W<T: std::fmt::Debug>(T);

impl<T: std::fmt::Debug> Drop for W<T> {
    fn drop(&mut self) {
        println!("{:?}", self.0);
    }
}

fn main() {
    let mut x = String::from("hi");
    let w = W(&x);
    x.push('!');
    println!("{}", x);
}
