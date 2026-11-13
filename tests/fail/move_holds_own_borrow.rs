// expect: L0311
// After `tie`, `d` carries the borrow of itself: moving it into a call is a
// use of its own lifetime, so the borrow is still live there (rustc E0505).
struct N<'a>(&'a u8);
fn tie<'a>(x: &'a mut N<'a>) -> &'a mut N<'a> {
    x
}
fn take<T>(_t: T) {}
fn main() {
    let z = 5u8;
    let mut d = N(&z);
    {
        let r = tie(&mut d);
        println!("{}", r.0);
    }
    take(d);
}
