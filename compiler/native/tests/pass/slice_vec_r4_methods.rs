// [u8] eq_ignore_ascii_case/is_ascii, rotate, partition_point, Vec extend_from_within, resize_with, dedup_by(_key) (R4).
#[derive(Debug, Clone, PartialEq)]
struct D(u32, &'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {} {}", self.0, self.1); } }
fn main() {
    let a = b"Hello".to_vec();
    println!("{} {} {}", a.eq_ignore_ascii_case(b"hELLO"), b"ab".eq_ignore_ascii_case(b"abc"), a[..2].eq_ignore_ascii_case(&b"HE"[..]));
    println!("{} {} {}", b"ab".is_ascii(), [0x80u8].is_ascii(), a.is_ascii());
    let mut v = vec![1, 2, 3, 4, 5];
    v.rotate_left(2); println!("{v:?}");
    v.rotate_right(1); println!("{v:?}");
    v[1..].rotate_right(4); println!("{v:?}");
    let mut arr = [1, 2, 3]; arr.rotate_left(3); arr.rotate_left(0); println!("{arr:?}");
    let s = [1, 2, 2, 4, 7, 9];
    println!("{} {} {}", s.partition_point(|&x| x < 4), s.partition_point(|_| true), s.partition_point(|_| false));
    let mut calls = 0; let p = s.partition_point(|&x| { calls += 1; x < 7 }); println!("{p} {calls}");
    let mut w = vec![String::from("a"), String::from("b"), String::from("c")];
    w.extend_from_within(1..); println!("{w:?} {}", w.capacity());
    w.extend_from_within(..=0); w.extend_from_within(..); println!("{w:?} {}", w.capacity());
    let mut n = vec![1u8, 2, 3]; n.extend_from_within(..2); println!("{n:?} {}", n.capacity());
    let mut k = 0;
    let mut r: Vec<i32> = Vec::new(); r.resize_with(3, || { k += 1; k * 10 }); println!("{r:?} {}", r.capacity());
    r.resize_with(1, || 0); println!("{r:?}");
    let mut dv = vec![D(1, "a"), D(1, "b"), D(2, "c"), D(2, "d"), D(1, "e")];
    dv.dedup_by(|a, b| { println!("cmp {} {}", a.1, b.1); a.0 == b.0 });
    println!("{:?}", dv.iter().map(|d| d.1).collect::<Vec<_>>());
    let mut kv = vec![10, 11, 20, 21, 30, 12];
    kv.dedup_by_key(|x| *x / 10); println!("{kv:?}");
    let mut dd = vec![D(5, "x"), D(6, "y"), D(7, "z")];
    dd.resize_with(1, || D(0, "new"));
    println!("after truncate");
    dd.resize_with(2, || D(9, "new"));
    println!("{:?}", dd.iter().map(|d| d.1).collect::<Vec<_>>());
}
