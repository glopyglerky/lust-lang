// A closure param annotation with the caller's lifetimes, passed for an
// `impl Fn` whose input holds the callee's type param below the top
// (`&mut V<'a, C>`, `&Option<C>`, `&U<C>`): the annotation pairs with the
// call's instantiation of `C` (R18, format/span.rs).
pub struct W<'t> { pub text: &'t str }
pub struct V<'a, C> { pub n: &'a u32, pub ctx: C }
pub struct U<C> { pub ctx: C }
pub fn new_v<'a, C>(visit: impl Fn(&mut V<'a, C>) -> usize, n: &'a u32, ctx: C) -> (usize, V<'a, C>) {
    let mut v = V { n, ctx };
    (visit(&mut v), v)
}
pub fn g<'a, 't>(w: &'a mut W<'t>, n: &'a u32) -> (usize, V<'a, &'a mut W<'t>>) {
    new_v(|v: &mut V<'a, &'a mut W<'t>>| v.ctx.text.len(), n, w)
}
fn opt<C>(f: impl Fn(&Option<C>) -> u32, ctx: C) -> u32 { f(&Some(ctx)) }
fn h<'a, 'b>(w: &'a &'b u32) -> u32 { opt(|v: &Option<&'a &'b u32>| **v.unwrap() + 1, w) }
fn uu<C, F: Fn(&U<C>) -> u32>(f: F, ctx: C) -> u32 { f(&U { ctx }) }
fn k<'a, 'b>(w: &'a &'b u32, keep: &std::cell::RefCell<Vec<&'b u32>>) -> u32 {
    uu(|v: &U<&'a &'b u32>| { keep.borrow_mut().push(*v.ctx); 2 }, w)
}
fn main() {
    let s = String::from("abc");
    let mut w = W { text: &s };
    let (k1, v) = g(&mut w, &1);
    println!("{} {}", k1, v.ctx.text.len() + *v.n as usize);
    let x = 7u32;
    let xr = &x;
    println!("{}", h(&xr));
    let keep = std::cell::RefCell::new(Vec::new());
    println!("{} {}", k(&xr, &keep), keep.borrow().len());
}
