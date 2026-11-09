// expect: L0733
// rustc E0521: the returned closure's argument is higher-ranked, so it cannot go into `F`'s `L<'s>`.
struct Q { n: u32 }
struct L<'a> { q: &'a Q, c: usize }
fn mk3<'s, F: Fn(&mut L<'s>)>(run: &'s F) -> impl Fn(&Q) + 's { move |x: &Q| { let mut l = L { q: x, c: 0 }; run(&mut l); } }
fn main() {}
