// ts-rust ls/crossproject.rs: a call of a fn that returns `!` relates nothing to its
// destination, here a closure output or return place with two regions.
fn nil() -> ! { panic!("nil") }
fn g<'a, 'b>(x: Option<&'b &'a u32>) -> &'b &'a u32 {
    if let Some(x) = x {
        return x;
    }
    nil()
}
fn run<'a>(o: Option<&&'a u32>) -> u32 {
    let d = o.unwrap_or_else(|| nil());
    **d
}
fn main() {
    let x = 4;
    let r = &x;
    println!("{} {}", g(Some(&r)), run(Some(&r)));
}
