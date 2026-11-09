// expect: L0306
// rustc E0597: the recursive call inside `chain`'s own returned closure passes `&inner` for `'s`.
struct Q { n: u32 }
struct L<'a> { q: &'a Q }
fn chain<'s, F: Fn(&mut L<'s>)>(run: &'s F, q: &'s Q) -> impl Fn() + 's { move || { let inner = Q { n: 8 }; let _ = chain(run, &inner); run(&mut L { q }); } }
fn main() {}
