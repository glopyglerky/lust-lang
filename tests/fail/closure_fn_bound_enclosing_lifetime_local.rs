// expect: L0733
// rustc: lifetime may not live long enough. `outer` is a local, but `F`'s bound wants an `L<'s>` from the closure.
struct Q { n: u32 }
struct L<'a> { q: &'a Q, c: usize }
fn call0(f: impl FnOnce()) { f() }
fn via<'v, G: Fn(usize, &mut L<'v>)>(g: &G, q: &'v Q) {
    let mut l = L { q, c: 5 };
    g(5, &mut l);
}
fn chain<'s, F>(run: &F, q: &'s Q, depth: usize)
where
    F: Fn(usize, &mut L<'s>),
{
    let outer = Q { n: 9 };
    let c = move || {
        let mut l = L { q: &outer, c: 1 };
        run(1, &mut l);
    };
    c();
}
fn main() {
    let q = Q { n: 3 };
    let run = |i: usize, l: &mut L<'_>| println!("{} {} {}", i, l.q.n, l.c);
    chain(&run, &q, 1);
}
