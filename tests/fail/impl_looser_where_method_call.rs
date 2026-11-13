// expect: L0612
// rustc E0277: the trait item's where clause holds at the call too.
trait Get {
    fn take<T>(&self, x: T, y: u8) -> usize
    where
        T: Copy;
}
struct A;
impl Get for A {
    fn take<T>(&self, _x: T, y: u8) -> usize {
        y as usize
    }
}
fn main() {
    let r = &&A;
    println!("{}", r.take(String::new(), 3));
}
