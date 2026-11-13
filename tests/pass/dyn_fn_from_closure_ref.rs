// A dyn Fn / FnMut value made from &C or &mut C of a closure (core's impl Fn* for &F / &mut F):
// an impl FnMut parameter forwarded as `&mut v` (ts-rust Node::for_each_child), and &v as &dyn Fn.
fn each_dyn(n: u32, v: &mut dyn FnMut(u32) -> bool) -> bool {
    for i in 0..n {
        if v(i) { return true; }
    }
    false
}
fn each(n: u32, mut v: impl FnMut(u32) -> bool) -> bool {
    each_dyn(n, &mut v)
}
fn each_shared(n: u32, v: impl Fn(u32) -> bool) -> bool {
    let d: &dyn Fn(u32) -> bool = &v;
    (0..n).any(|i| d(i))
}
fn main() {
    let mut seen = Vec::new();
    let found = each(5, &mut |i| { seen.push(i); i == 3 });
    println!("{found} {seen:?}");
    let mut count = 0;
    let mut f = |i: u32| { count += i; false };
    each(4, &mut f);
    each(2, &mut f);
    println!("{count}");
    let lim = 2;
    println!("{}", each_shared(3, &|i: u32| i > lim));
    println!("{}", each(3, |i| i == 1));
}
