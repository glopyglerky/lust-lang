// The counters of Take, Skip, and Enumerate after borrowing consumers,
// as std moves them (W-STD-B): `find`, `position`, `any`, and `all` run
// through `try_fold` (a Take counts only the items its inner yields),
// `nth` through each adapter's own `nth` or core's `advance_by` then
// `next`. Visible through `size_hint` when the inner runs out early.
struct W {
    k: u32,
}

impl Iterator for W {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.k < 2 {
            self.k += 1;
            Some(self.k)
        } else {
            None
        }
    }
}

fn main() {
    let mut it = (W { k: 0 }).take(5);
    println!("{:?} {:?}", it.find(|&x| x > 10), it.size_hint());
    let mut it = (W { k: 0 }).take(5);
    println!("{:?} {:?}", it.position(|x| x > 10), it.size_hint());
    let mut it = (W { k: 0 }).take(5);
    println!("{:?} {:?}", it.any(|x| x > 10), it.size_hint());
    let mut it = (W { k: 0 }).take(5);
    println!("{:?} {:?}", it.all(|x| x < 10), it.size_hint());
    let mut it = (W { k: 0 }).take(5);
    println!("{:?} {:?}", it.nth(7), it.size_hint());
    let mut it = (W { k: 0 }).take(5);
    println!("{:?} {:?}", it.nth(1), it.size_hint());
    let mut it = (W { k: 0 }).map(|x| x + 1).take(5);
    println!("{:?} {:?}", it.find(|&x| x > 10), it.size_hint());
    let mut it = (W { k: 0 }).take(3).skip(1).map(|x| x * 2);
    println!("{:?} {:?}", it.nth(2), it.size_hint());
    let mut it = (W { k: 0 }).take(4).skip(1);
    println!("{:?} {:?}", it.find(|_| false), it.size_hint());
    let mut it = (W { k: 0 }).take(9).enumerate();
    println!("{:?} {:?}", it.nth(4), it.size_hint());
    let mut it = (W { k: 0 }).chain(W { k: 1 }.take(6));
    println!("{:?} {:?}", it.nth(5), it.size_hint());
    let mut it = (W { k: 0 }).take(6).filter(|x| x % 2 == 1);
    println!("{:?} {:?}", it.nth(3), it.size_hint());
    // Filter's `next` is `find` on its inner: a Take under it counts
    // only yielded items, also on the last, empty pull.
    let mut it = (W { k: 0 }).take(3).filter(|x| *x > 1).map(|x| x * 10);
    println!("{:?} {:?}", it.nth(2), it.size_hint());
    println!("{:?} {:?}", it.next(), it.size_hint());
    let mut it = (W { k: 0 }).take(4).filter_map(|x| if x > 5 { Some(x) } else { None });
    println!("{:?} {:?}", it.next(), it.size_hint());
    let mut it = (W { k: 0 }).take(4).skip_while(|&x| x < 9);
    println!("{:?} {:?}", it.next(), it.size_hint());

    let mut it = "a,b".split(',').take(5);
    println!("{:?} {:?}", it.find(|_| false), it.size_hint());
    let mut it = "a,b".split(',').take(5);
    println!("{:?} {:?}", it.nth(9), it.size_hint());
    let mut it = "a b c".split_whitespace().take(7);
    println!("{:?} {:?}", it.position(|s| s == "z"), it.size_hint());
    let v = [1u8, 2, 3];
    let mut it = v.iter().take(8).copied();
    println!("{:?} {:?}", it.nth(4), it.size_hint());
    let mut it = "x,y,z".split(',').take(6).peekable();
    let first = match it.peek() {
        Some(s) => s.len(),
        None => 0,
    };
    println!("{:?} {:?} {:?}", first, it.nth(5), it.size_hint());
}
