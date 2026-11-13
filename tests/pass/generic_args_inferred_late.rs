// Generic arguments fixed after the call (by a later use or by the
// return type), and `?Sized` parameters at unsized types.
trait L {
    fn l(&self) -> usize;
}

impl L for str {
    fn l(&self) -> usize {
        self.len()
    }
}

fn g<T: L + ?Sized>(t: &T) -> usize {
    t.l()
}

fn make<T: Default>() -> T {
    T::default()
}

fn first<T: Copy>(v: &[T]) -> T {
    v[0]
}

fn main() {
    println!("{}", g("four"));
    let x: u64 = make();
    println!("{}", x);
    println!("{}", first(&[3i8, 4]));
    let d = make();
    let e: u8 = d;
    println!("{}", e);
}
