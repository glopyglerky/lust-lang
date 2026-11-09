// The item type of a closure-bound iterator (FromFn, a user wrapper)
// flows into the closure's integer literal state, as in rustc.
struct F2<F>(F);
impl<T, F: FnMut() -> Option<T>> Iterator for F2<F> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        (self.0)()
    }
}
fn mk<T, F: FnMut() -> Option<T>>(f: F) -> F2<F> {
    F2(f)
}
fn main() {
    let d: Vec<u32> = std::iter::from_fn({
        let mut n = 0;
        move || {
            n += 1;
            if n < 5 { Some(n) } else { None }
        }
    })
    .collect();
    let mut m = 0;
    let e: Vec<u64> = mk(move || {
        m += 1;
        if m < 4 { Some(m * 1_000_000_000_000) } else { None }
    })
    .collect();
    let mut k = 250;
    let s: u8 = std::iter::from_fn(move || {
        k += 1;
        if k < 255 { Some(k) } else { None }
    })
    .map(|x| x - 250)
    .sum();
    let v = std::iter::from_fn(|| Some(1)).take(3).collect::<Vec<_>>();
    println!("{:?} {:?} {} {:?} {}", d, e, s, v, v.capacity());
}
