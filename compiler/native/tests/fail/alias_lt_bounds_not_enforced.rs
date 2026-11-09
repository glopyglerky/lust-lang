// expect: L0733
// rustc: bounds on alias params are not enforced, so `'y: 'x` is not
// known in `f`.
type R<'a, 'b: 'a> = (&'a str, &'b str);
fn f<'x, 'y>(p: R<'x, 'y>) -> &'x str { p.1 }
fn main() { let a = String::from("a"); let b = String::from("b"); println!("{}", f((&a, &b))); }
