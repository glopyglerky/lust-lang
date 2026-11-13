// expect: L0733
// rustc E0521: the annotation says the
// `C` inside `&Option<C>` is `&'static u32`, the call binds `&'b u32`.
fn opt<C>(f: impl Fn(&Option<C>) -> u32, ctx: C) -> u32 { f(&Some(ctx)) }
fn h<'b>(w: &'b u32) -> u32 { opt(|v: &Option<&'static u32>| *v.unwrap(), w) }
fn main() { let x = 3u32; println!("{}", h(&x)); }
