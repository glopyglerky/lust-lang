// expect: L0110
// rustc: defaults for generic parameters are not allowed here (an impl).
struct W<T>(T);

impl<T = i32> W<T> {
    fn get(self) -> T {
        self.0
    }
}

fn main() {
    println!("{}", W(1).get());
}
