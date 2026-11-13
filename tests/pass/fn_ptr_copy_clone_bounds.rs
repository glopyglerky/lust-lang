// Fn pointers (any signature, higher-ranked too) and fn items meet T: Copy
// and T: Clone bounds, as core's builtin impls give them.
fn foo(x: &u8) -> u8 {
    *x + 1
}
fn twice<T: Copy>(t: T) -> (T, T) {
    (t, t)
}
fn cloned<T: Clone>(t: &T) -> T {
    t.clone()
}
trait Dup {
    fn dup(&self) -> (Self, Self)
    where
        Self: Sized;
}
impl<T: Copy> Dup for T {
    fn dup(&self) -> (T, T) {
        (*self, *self)
    }
}
fn main() {
    let p: fn(&u8) -> u8 = foo;
    let (a, b) = twice(p);
    println!("{} {}", a(&1), b(&2));
    let hr: for<'a> fn(&'a str) -> &'a str = |s| s;
    println!("{}", cloned(&hr)("hr"));
    let (f, g) = twice(foo);
    println!("{} {}", f(&3), g(&4));
    let (x, _) = (|n: u8| n * 2).dup();
    println!("{}", x(5));
    let two: fn(u8, &str) -> usize = |a, s| a as usize + s.len();
    println!("{}", twice(two).1(7, "ab"));
    let v = vec![p, p];
    println!("{}", cloned(&v)[1](&6));
}
