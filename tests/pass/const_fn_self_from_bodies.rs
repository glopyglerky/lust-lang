// A const fn body that a method body evaluates (array lengths, const
// blocks) reads `Self` as the fn's own impl type, not the caller's
// (it read Q::N for P::len(): 25, rustc 22).
struct P;
impl P {
    const N: usize = 2;
    const fn len() -> usize {
        Self::N
    }
    const fn nested() -> usize {
        Self::len() + Q::len()
    }
}
struct Q;
impl Q {
    const N: usize = 5;
    const fn len() -> usize {
        Self::N
    }
    fn arr(&self) -> usize {
        let a = [0u8; P::len()];
        let b = [0u8; Self::len()];
        let c = const { P::nested() };
        a.len() * 100 + b.len() * 10 + c
    }
}
fn generic<T>(_t: T) -> usize {
    let a = [1u8; P::nested()];
    a.len()
}
fn main() {
    println!("{} {}", Q.arr(), generic(1u8));
}
