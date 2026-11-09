// A body that calls a generic `impl Fn` fn stays pending in `lust check` (no instances): its reads are kept, so a derive delta that moves none of them skips it, and one that moves one (S: Copy in `user_s`) checks it again.
#[derive(Clone, Copy, Debug)]
pub struct S { x: u32 }
const ROUTE: u8 = 1;
pub struct T { y: u8 }
fn mk<V: Copy>(v: V) -> impl Fn() -> V { move || v }
fn user_t(t: &T) -> u8 { let f = mk(t.y); f() }
fn user_s(s: S) -> u32 { let f = mk(s); let a = f(); let b = f(); a.x + b.x }
fn main() {
    let _r = ROUTE;
    let s = S { x: 3 };
    println!("{} {} {:?}", user_t(&T { y: 2 }), user_s(s), s);
}
