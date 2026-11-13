// expect: L0110
// rustc: defaults for generic parameters are not allowed here (a method).
struct W;

impl W {
    fn get<T = u8>(&self, x: T) -> T {
        x
    }
}

fn main() {
    println!("{}", W.get(3));
}
