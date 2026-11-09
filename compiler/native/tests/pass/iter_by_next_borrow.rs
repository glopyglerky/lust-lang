// A user iterator under the borrowing consumers (`find`, `nth`,
// `position`, `any`, `all`, `size_hint`) stays where it is (W-STD-B):
// it is usable after the call and drops at its own scope end; a
// temporary drops at the end of its statement.
struct N(u32);
impl Iterator for N {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < 6 { self.0 += 1; Some(self.0) } else { None }
    }
}
impl Drop for N {
    fn drop(&mut self) { println!("drop N at {}", self.0); }
}
struct Words<'a>(&'a str);
impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let s: &'a str = self.0;
        let s = s.trim_start();
        if s.is_empty() { return None; }
        let end = s.as_bytes().iter().position(|b| *b == b' ').unwrap_or(s.len());
        self.0 = &s[end..];
        Some(&s[..end])
    }
}
struct Holder { it: N, tag: u8 }
fn first_two<I: Iterator<Item = u32>>(mut i: I) -> (Option<u32>, Option<u32>) {
    let a = i.find(|x| *x > 1);
    (a, i.next())
}
fn main() {
    println!("{:?}", N(0).find(|x| *x == 2));
    println!("after temp");
    let mut h = Holder { it: N(1), tag: 7 };
    println!("{:?} {:?} {}", h.it.position(|x| x == 4), h.it.next(), h.tag);
    println!("{:?}", first_two(N(0)));
    println!("{:?}", first_two(vec![1u32, 5, 9].into_iter()));
    let text = String::from("a bb ccc dd");
    let mut w = Words(&text);
    let long = w.find(|s| s.len() > 1);
    let next = w.next();
    println!("{:?} {:?} {}", long, next, w.all(|s| s.len() == 2));
    let n = N(3);
    println!("{:?}", n.size_hint());
    println!("end");
}
