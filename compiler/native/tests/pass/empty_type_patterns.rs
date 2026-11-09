enum Void {}
struct HasVoid(Void, u8);
enum W {
    A(u8),
    B(Void),
}
fn a(r: Result<u8, std::convert::Infallible>) -> u8 {
    let Ok(x) = r;
    x
}
fn b(o: Option<Void>) -> u8 {
    match o {
        None => 1,
    }
}
fn c(v: Void) -> u8 {
    match v {}
}
fn d(w: W) -> u8 {
    match w {
        W::A(n) => n,
    }
}
fn e(t: (Void, u8)) -> u8 {
    match t {}
}
fn f(x: HasVoid) -> u8 {
    match x {}
}
fn g(o: Option<Void>) -> u8 {
    let None = o;
    7
}
fn main() {
    println!("{}", a(Ok(3)));
    println!("{}", b(None));
    println!("{} {}", d(W::A(9)), g(None));
    let _c: fn(Void) -> u8 = c;
    let _e: fn((Void, u8)) -> u8 = e;
    let _f: fn(HasVoid) -> u8 = f;
    println!("ok");
}
