// Tool attributes are inert: rustc accepts rustfmt::* and clippy::*
// names and change no output, so lust accepts and ignores them too.
#[rustfmt::skip]
fn pick() -> i64 { 3 }
#[rustfmt::skip]
struct Pair {
    #[rustfmt::skip]
    a: i64,
    b: i64,
}
#[rustfmt::skip::attributes(custom)]
fn other() -> i64 { 4 }
fn main() {
    #[rustfmt::skip]
    let p = Pair { a: 1, b: 2 };
    let total = match p.a {
        #[rustfmt::skip]
        1 => pick() + other(),
        _ => 0,
    };
    println!("{} {}", p.b, total);
}
