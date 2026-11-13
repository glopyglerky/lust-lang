// A `for` loop's iterator drops before the temporaries of its head
// expression, as rustc's `match into_iter(head) { mut iter => loop {..} }`
// does: the iterator may borrow them until it drops (ts-rust
// goport_multiprog.rs: an RPIT iterator zipped with `order().iter()`).
struct T(&'static str);
impl Drop for T { fn drop(&mut self) { println!("drop {}", self.0); } }
struct It<'a>(&'a T, u32);
impl Iterator for It<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<u32> { self.1 += 1; if self.1 < 3 { Some(self.1) } else { None } }
}
impl Drop for It<'_> { fn drop(&mut self) { println!("drop it of {}", self.0 .0); } }
fn mk(t: &T) -> It<'_> { It(t, 0) }
fn files() -> impl Iterator<Item = usize> { 0..2 }
fn main() {
    for x in mk(&T("head")) { println!("x {}", x); }
    let b = vec![10usize, 20];
    let mut s = 0;
    for (x, &id) in files().zip(b.clone().iter()) { s += x + id; }
    for (x, y) in mk(&T("zip")).zip(mk(&T("zip2"))) { println!("{} {}", x, y); }
    let mut n = 0;
    for x in mk(&T("brk")) { n += x; if x == 1 { break; } }
    println!("{} {}", s, n);
}
