// `#[expect(lint)]` works as `#[allow(lint)]` does: it silences the
// lint (also the deny-by-default ones lust checks), and an expectation
// the code does not meet is only a warning.
#![expect(unused)]
#[expect(dead_code, reason = "never called")]
fn never() {}
struct S {
    #[expect(dead_code)]
    a: u8,
}
fn f(#[expect(unused_variables)] x: u8) -> u8 {
    7
}
#[expect(arithmetic_overflow, unconditional_panic)]
fn wraps() -> u8 {
    let x: u8 = 200;
    x.wrapping_add(100)
}
fn main() {
    #[expect(unused_variables)]
    let y = 1;
    let _ = S { a: 1 };
    #[expect(clippy::all, no_such_lint)]
    let z = f(2) + wraps();
    println!("{z}");
}
