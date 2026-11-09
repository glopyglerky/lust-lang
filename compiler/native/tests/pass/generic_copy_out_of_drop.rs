// A Copy-bounded generic field copies out of a type that implements
// Drop (no move, so no E0509).
struct W<T>(T, u8);

impl<T> Drop for W<T> {
    fn drop(&mut self) {
        println!("drop W {}", self.1);
    }
}

fn get<T: Copy>(w: W<T>) -> T {
    w.0
}

fn main() {
    println!("{}", get(W(5u8, 1)));
}
