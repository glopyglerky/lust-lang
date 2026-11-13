// expect: L0733
// rustc E0621: `x` must live for `'s` to go into `F`'s `L<'s>`. The fn returns `impl Trait + 's`, whose
// signature is an opaque one (lust accepted this before).
struct Q { n: u32 }
struct L<'a> { q: &'a Q, c: usize }
fn mk5<'s, F: Fn(&mut L<'s>)>(run: &'s F, x: &Q) -> impl Fn() + 's { let mut l = L { q: x, c: 0 }; run(&mut l); move || {} }
fn main() {}
