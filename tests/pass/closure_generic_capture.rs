// A closure in a generic fn captures as the generic body says: `T`
// is not known to be Copy there, so `let y = t` moves `t` in and the
// closure is FnOnce, even when `T` is `u8` at the call.
fn size<T>(_x: &T) -> usize {
    std::mem::size_of::<T>()
}
fn keep<T>(_t: T) {}
fn wrap<T>(t: T) -> usize {
    let c = || {
        let y = t;
        keep(y);
    };
    let n = size(&c);
    c();
    n
}
fn borrow<T: Copy>(t: T) -> usize {
    let c = || {
        let y = t;
        keep(y);
    };
    let n = size(&c);
    c();
    c();
    n
}
fn main() {
    println!("{} {}", wrap(1u8), wrap(2u32));
    println!("{} {}", borrow(1u8), borrow(2u32));
}
