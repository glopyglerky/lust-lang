// Valid shapes next to the round 5 false accepts: a dyn Fn's written
// `'static` Adt slot, closures under a call's expected result, fn values
// whose type argument fills a pointer's output, and closures returned as
// `impl Fn*` (tail, block, return, let-bound with annotations).
static G: u8 = 9;
#[derive(Debug)]
pub struct W<'a, X>(&'a u8, X);
fn ap<U, F: FnOnce() -> U>(f: F) -> U {
    f()
}
fn take<T>(f: fn(&u8) -> T, x: &u8) -> T {
    f(x)
}
fn tk<T, F: Fn(&u8) -> T>(f: F, x: &u8) -> T {
    f(x)
}
fn st(_a: &u8) -> &'static u8 {
    &G
}
fn cp(a: &u8) -> u8 {
    *a
}
fn pair(p: &(u8, u8)) -> (&u8, &u8) {
    (&p.0, &p.1)
}
struct S<K> {
    f: fn(&(K, u8)) -> (&K, &u8),
}
fn pair_str<'a>(p: &'a (&'static str, u8)) -> (&'a &'static str, &'a u8) {
    (&p.0, &p.1)
}
fn vec_ref<'a>(v: &'a Vec<u8>) -> &'a Vec<u8> {
    ap(|| &v)
}
fn str_or<'a>(s: &'a String) -> Option<&'a str> {
    Some(1u8).map(|k| if k == 1 { s } else { "lit" })
}
fn slice<'a>(v: &'a Vec<u8>) -> Option<&'a [u8]> {
    Some(1u8).map(|_| &v[..])
}
fn mk_id() -> impl Fn(&u8) -> &u8 {
    |a| a
}
fn mk_static() -> impl Fn(&u8) -> &'static u8 {
    |_a| &G
}
fn mk_pair() -> impl FnMut(&u8) -> (u8, &'static u8) {
    {
        |a| (*a, &G)
    }
}
fn mk_ret() -> impl FnOnce(&u8) -> Option<&u8> {
    return |a| Some(a);
}
fn mk_let() -> impl Fn(&u8) -> u8 {
    let f = |a: &u8| *a + 1;
    f
}
fn mk_let_static() -> impl Fn(&u8) -> &'static u8 {
    let f = |_a: &u8| &G;
    f
}
fn mk_gen<T: Copy>(t: T) -> impl Fn(&u8) -> (T, &u8) {
    move |a| (t, a)
}
fn main() {
    let f: Box<dyn Fn(&u8, usize) -> (W<'static, &u8>, &u8)> = Box::new(|a, _| (W(&G, a), a));
    let y = 2u8;
    println!("{:?}", f(&y, 0));
    let v = vec![1u8, 2];
    let s = String::from("x");
    println!("{:?} {:?} {:?}", vec_ref(&v), str_or(&s), slice(&v));
    let x = 4u8;
    println!("{} {} {}", take(st, &x), take(cp, &x), tk(st, &x));
    let p = (1u8, 2u8);
    let q: fn(&(u8, u8)) -> (&u8, &u8) = pair;
    println!("{:?}", q(&p));
    let sp = S { f: pair_str };
    let e = ("k", 3u8);
    println!("{:?}", (sp.f)(&e));
    let r = {
        let k = 2u8;
        mk_static()(&k)
    };
    println!("{} {}", mk_id()(&x), r);
    let mut m = mk_pair();
    println!("{:?} {:?}", m(&x), mk_ret()(&x));
    println!("{} {} {:?}", mk_let()(&x), mk_let_static()(&x), mk_gen(7u16)(&x));
}
