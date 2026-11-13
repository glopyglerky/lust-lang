// expect: L0306
// rustc E0597: `chain`'s bound `F: Fn(&mut L<'s>)` makes the call's `'s` the caller's, so `inner` must
// live for it. `chain` returns `impl Fn() + 's` (an opaque region signature); lust skipped its bounds.
struct Q { n: u32 }
struct L<'a> { q: &'a Q }
fn chain<'s, F: Fn(&mut L<'s>)>(q: &'s Q, run: &'s F) -> impl Fn() + 's {
    move || run(&mut L { q })
}
fn user<'s, F: Fn(&mut L<'s>)>(run: &'s F) {
    let inner = Q { n: 8 };
    let g = chain(&inner, run);
    g();
}
fn main() {
    let q = Q { n: 1 };
    let run = |l: &mut L<'_>| println!("{}", l.q.n);
    user(&run);
    let _ = &q;
}
