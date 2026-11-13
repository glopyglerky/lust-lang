// Closure templates keep the reads of their last check: a derive toggle checks one again only when a read moved (Copy, PartialEq), not when its body names the type.
#[derive(Clone, Debug, PartialEq)]
pub struct S { x: u32 }
const ROUTE: u8 = 1;
fn keep<T>(s: &S, t: T) -> (u32, T) { let f = |v: T| v; (s.x, f(t)) }
fn cap<T: Copy>(s: S, t: T) -> (u32, T) { let f = move || s; let a = f(); let b = s; (a.x + b.x, t) }
fn eqc<T>(s: &S, t: T) -> (bool, T) { let f = |a: &S| *a == S { x: 2 }; (f(s), t) }
fn main() {
    let _r = ROUTE;
    let s = S { x: 2 };
    let (a, n) = keep(&s, ROUTE);
    let (e, k) = eqc(&s, 4u8);
    let (b, m) = cap(s.clone(), 3u8);
    println!("{:?} {} {} {} {} {} {}", s, a, n, b, m, e, k);
}
