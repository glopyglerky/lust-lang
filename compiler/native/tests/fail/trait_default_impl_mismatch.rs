// expect: L0110
// rustc E0053: `impl Get for X` means `Get<u8>`, so `get` must return u8.
trait Get<T = u8> {
    fn get(&self) -> T;
}

struct X;

impl Get for X {
    fn get(&self) -> u16 {
        9
    }
}

fn main() {
    println!("{}", X.get());
}
