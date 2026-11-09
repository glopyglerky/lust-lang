// Generic fn items coerced to fn pointers whose regions are `'static`
// or a lifetime of the enclosing fn: the type param takes that region.
#[derive(Debug)]
struct W<T>(T);
fn id<T>(x: T) -> T {
    x
}
fn wrap<T>(x: T) -> W<T> {
    W(x)
}
fn idlt<'a, T>(x: &'a T) -> &'a T {
    x
}
fn run<'a>(v: &'a u8) -> W<&'a u8> {
    let g: fn(&'a u8) -> W<&'a u8> = wrap;
    let h: fn(&u8) -> &u8 = idlt;
    let k: fn(&'a u8) -> &'a u8 = id;
    println!("{} {}", h(v), k(v));
    g(v)
}
fn main() {
    let s: fn(&'static str) -> &'static str = id;
    let n: fn(u8) -> W<u8> = wrap;
    let x = 5u8;
    println!("{} {:?} {:?}", s("hi"), n(3), run(&x));
}
