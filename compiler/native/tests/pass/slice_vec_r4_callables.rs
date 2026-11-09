// dedup_by, dedup_by_key and partition_point with fn items, fn pointers and &mut closures (R4).
fn same(a: &mut i32, b: &mut i32) -> bool { a == b }
fn small(x: &i32) -> bool { *x < 3 }
fn key(x: &mut i32) -> i32 { *x / 10 }
fn main() {
    let mut v = vec![1, 1, 2, 12, 15, 30]; v.dedup_by(same); println!("{v:?}");
    let mut f = |a: &mut i32, b: &mut i32| a == b; let mut w = vec![4, 4, 5]; w.dedup_by(&mut f); println!("{w:?}");
    v.dedup_by_key(key); println!("{v:?}");
    let p: fn(&i32) -> bool = small; println!("{} {}", v.partition_point(small), v.partition_point(p));
}
