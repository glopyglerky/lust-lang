// Fat raw pointers as plain Copy values (tf-fatptr): struct fields,
// Option, tuples, arrays, Clone, generic sized-only helpers.
#[derive(Clone, Copy)]
struct H {
    p: *const [u8],
    q: Option<*const str>,
}
fn same_addr<T: ?Sized>(a: *const T, b: *const T) -> bool {
    a.cast::<u8>() == b.cast::<u8>()
}
fn main() {
    let v = vec![9u8; 4];
    let h = H { p: &v[..], q: Some("ab") };
    let h2 = h;
    let k = h.clone();
    let arr = [h, h2, k];
    println!("{} {}", arr.iter().map(|x| x.p.len()).sum::<usize>(), h2.q.is_some_and(|s| !s.is_null()));
    let t = (h.p, 7u8);
    let t2 = t;
    println!("{} {}", t2.0 == h.p, t2.1);
    let w = [1, 2];
    println!("{} {}", same_addr(&w[..] as *const [i32], &w[..1]), same_addr("a" as *const str, "a"));
    let mut o: Option<*const [u8]> = Some(&v[1..]);
    let taken = o.take();
    println!("{:?} {}", taken.map(|p| p.len()), o.is_none());
}
