// expect: L0617
// rustc E0309: a `let` annotation must be well-formed under the fn's bounds.
fn f<'a, T>(_z: &'a ()) -> usize {
    let x: Option<&'a T> = None;
    x.map_or(0, |_| 1)
}
fn main() { println!("{}", f::<u8>(&())); }
