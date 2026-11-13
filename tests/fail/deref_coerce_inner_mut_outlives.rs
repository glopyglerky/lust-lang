// expect: L0733
// rustc: the reborrow behind &'a lives only for 'a, not 'b.
fn f<'a, 'b, T>(x: &'a &'b mut T) -> &'b T {
    x
}
fn main() {
    let mut v = 1;
    let m = &mut v;
    println!("{}", f(&m));
}
