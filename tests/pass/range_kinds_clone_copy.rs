// core 1.99: RangeTo, RangeToInclusive, and RangeFull are Copy; every range
// kind is Clone (RangeFrom too), in derives and by value.
#[derive(Clone, Copy)]
struct C { a: std::ops::RangeTo<u8>, b: std::ops::RangeToInclusive<i64>, c: std::ops::RangeFull }
#[derive(Clone)]
struct D { a: std::ops::RangeFrom<u16>, b: std::ops::RangeTo<u32> }
fn main() {
    let c = C { a: ..4, b: ..=-2, c: .. };
    let d = c;
    let e = c;
    println!("{} {} {}", d.a.end, e.b.end, c.a.end);
    let x = D { a: 3.., b: ..9 };
    let y = x.clone();
    drop(x);
    let mut it = y.a.clone();
    println!("{:?} {} {}", it.next(), y.b.end, y.a.start);
    let r = ..7u8;
    let s = r;
    println!("{} {}", r.end, s.end);
}
