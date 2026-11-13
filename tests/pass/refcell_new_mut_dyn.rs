// `RefCell::new(o)` of `o: &mut (dyn FnMut(..) + '_)`, called through the cell in a closure
// (ts-rust effect/ls/refactors/async_await_to_gen.rs transform_body_await_to_yield).
use std::cell::RefCell;
struct Tracker { k: u32 }
type TransformAwaitExpr<'f> = dyn FnMut(&Tracker, u32) -> u32 + 'f;
fn visit(n: u32, f: &mut dyn FnMut(u32) -> u32) -> u32 { f(n) + f(n + 1) }
fn transform(tracker: &Tracker, body: u32, on_await: &mut TransformAwaitExpr<'_>) -> u32 {
    let on_await = RefCell::new(on_await);
    let mut cl = |n: u32| (*on_await.borrow_mut())(tracker, n);
    visit(body, &mut cl)
}
fn main() {
    let t = Tracker { k: 3 };
    let mut total = 0;
    let mut f = |tr: &Tracker, n: u32| { total += n; n * tr.k };
    println!("{}", transform(&t, 2, &mut f));
    println!("{}", total);
}
