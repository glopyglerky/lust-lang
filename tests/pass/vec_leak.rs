// `Vec::leak` as a method, by path (ts-rust scanner_util.rs
// `Vec::leak(compute_ecma_line_starts(..))`), and as a `map` value: the
// buffer stays allocated, so the slice lives for `'static`. Each leaked
// slice is kept in a static, so the leak checker sees it reachable.
use std::sync::Mutex;
static KEPT: Mutex<Vec<usize>> = Mutex::new(Vec::new());
fn keep_ptr<T>(s: &[T]) {
    KEPT.lock().unwrap().push(s.as_ptr() as usize);
}
fn keep(s: &'static [i32]) -> usize {
    keep_ptr(s);
    s.len()
}
fn lk<'a, T>(v: Vec<T>) -> &'a mut [T] {
    Vec::leak(v)
}
fn main() {
    let s: &'static mut [i32] = vec![1, 2, 3].leak();
    s[0] = 9;
    keep_ptr(s);
    println!("{:?} {}", s, keep(Vec::leak(vec![4, 5])));
    let t = lk(vec![String::from("a"), String::from("b")]);
    t[1].push('!');
    keep_ptr(t);
    for x in t.iter() {
        keep_ptr(x.as_bytes());
    }
    println!("{:?}", t);
    let vs = vec![vec![1u8], vec![2, 3]];
    let ss: Vec<&mut [u8]> = vs.into_iter().map(Vec::leak).collect();
    for x in ss.iter() {
        keep_ptr(x);
    }
    println!("{:?}", ss);
    let mut v = Vec::with_capacity(10);
    v.push(1u64);
    let e: &[u64] = Vec::<u64>::leak(v);
    keep_ptr(e);
    println!("{:?} {}", e, Vec::<u8>::new().leak().len());
}
