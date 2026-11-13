// A body annotation `Box<dyn Fn()>` infers its object lifetime, so a
// borrowing closure fits while the box stays local; a written `+ '_` too.
fn main() {
    let s = String::from("hello");
    let b: Box<dyn Fn() -> usize> = Box::new(|| s.len());
    println!("{}", b());
    second();
}
trait T {
    fn v(&self) -> usize;
}
struct R<'a>(&'a str);
impl<'a> T for R<'a> {
    fn v(&self) -> usize {
        self.0.len()
    }
}
fn second() {
    let s = String::from("abcd");
    let b: Box<dyn T + '_> = Box::new(R(&s));
    println!("{}", b.v());
}
