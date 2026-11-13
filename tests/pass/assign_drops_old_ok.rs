// The drop of the old value at a whole write ends its borrows: after the
// write, the local holds only the new value. A value moved out first
// drops nothing at the write. Plain references and `Vec<&T>` have no
// drop that reads the borrow.
struct S<'a>(&'a i32);
impl Drop for S<'_> {
    fn drop(&mut self) {
        println!("drop S {}", self.0);
    }
}
fn id<T>(t: T) -> T {
    t
}
fn main() {
    let mut x = 1;
    let y = 2;
    let mut h = S(&x);
    h = S(&y);
    x += 1;
    println!("x {}", x);
    let mut k = S(&x);
    drop(k);
    x += 1;
    k = S(&y);
    println!("x {}", x);
    let mut m = Some(S(&y));
    m = id(m);
    let mut v = vec![&x];
    v = Vec::new();
    x += 1;
    let mut b: Box<dyn Fn() -> i32 + '_> = Box::new(|| x);
    println!("{}", b());
    b = Box::new(|| y);
    println!("{} {} {}", b(), k.0, m.is_some());
    let c = 5;
    let d: Box<dyn Fn() -> i32> = Box::new(move || c);
    let r = &d;
    println!("{} {}", r(), v.len());
}
