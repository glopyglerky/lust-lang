// repeat_n over a Clone + Drop element, as core's RepeatN: each item but
// the last is a clone, the last moves the element out; nth/skip advance
// with no clones; last and count make none; a zero count drops the
// element at once; a dropped iterator drops the element once.
struct D(i32);
impl Clone for D {
    fn clone(&self) -> D {
        println!("clone D({})", self.0);
        D(self.0 + 10)
    }
}
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D({})", self.0);
    }
}

fn main() {
    println!("-- collect");
    let v: Vec<D> = std::iter::repeat_n(D(1), 3).collect();
    println!("{:?}", v.iter().map(|d| d.0).collect::<Vec<_>>());
    drop(v);
    println!("-- zero");
    let it = std::iter::repeat_n(D(2), 0);
    println!("made {}", it.len());
    drop(it);
    println!("-- next, drop rest");
    let mut it = std::iter::repeat_n(D(3), 3);
    let a = it.next().map(|d| d.0);
    println!("{a:?} left {}", it.len());
    drop(it);
    println!("-- nth");
    let mut it = std::iter::repeat_n(D(4), 4);
    println!("{:?}", it.nth(2).map(|d| d.0));
    println!("{:?} {:?}", it.next().map(|d| d.0), it.next().map(|d| d.0));
    println!("-- nth past the end");
    let mut it = std::iter::repeat_n(D(5), 2);
    println!("{:?}", it.nth(5).map(|d| d.0));
    println!("left {}", it.len());
    println!("-- last and count");
    println!("{:?}", std::iter::repeat_n(D(6), 3).last().map(|d| d.0));
    println!("{}", std::iter::repeat_n(D(7), 3).count());
    println!("-- skip, rev, take");
    for d in std::iter::repeat_n(D(8), 4).skip(2).rev().take(1) {
        println!("item {}", d.0);
    }
    println!("-- String capacity: clones are fresh, the last is the original");
    let caps: Vec<usize> = std::iter::repeat_n(String::with_capacity(16) + "ab", 3).map(|s| s.capacity()).collect();
    println!("{caps:?}");
    let s: String = std::iter::repeat_n(String::from("xy"), 3).collect();
    println!("{s}");
}
