// `S<'a, 'b> { x: &'a mut W<'b>, y: &'b mut W<'a> }` implies `'a == 'b` for every use: rustc
// infers the outlives requirements of a struct's fields.
struct W<'c> { v: &'c u8 }
struct S<'a, 'b> { x: &'a mut W<'b>, y: &'b mut W<'a> }
fn f<'a, 'b>(s: S<'a, 'b>) -> &'a mut W<'a> { s.x }
fn main() {
    let (u1, u2) = (1u8, 2u8);
    let mut wa = W { v: &u1 };
    let mut wb = W { v: &u2 };
    let s = S { x: &mut wb, y: &mut wa };
    println!("{}", f(s).v);
}
