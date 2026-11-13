// rustc compares literal tokens in a macro matcher by their written
// text, not their value: `0x1` is not `1`, `'\x61'` is not `'a'`,
// `r"a"` is not `"a"`. A doc comment is the raw string `r" a"`.
macro_rules! m { (1) => { "one" }; ($t:tt) => { "other" }; }
macro_rules! u { (1u8) => { "u8" }; ($t:tt) => { "other" }; }
macro_rules! c { ('a') => { "a" }; ($t:tt) => { "other" }; }
macro_rules! s { ("a") => { "a" }; ($t:tt) => { "other" }; }
macro_rules! r { (r"a") => { "raw" }; ($t:tt) => { "other" }; }
macro_rules! f { (1.0) => { "f" }; ($t:tt) => { "other" }; }
macro_rules! b { (b"a") => { "b" }; ($t:tt) => { "other" }; }
macro_rules! d { (#[doc = r" a"]) => { "doc" }; ($($t:tt)*) => { "other" }; }
macro_rules! x { (0x1) => { "hex" }; ($t:tt) => { "other" }; }
// Forwarded tokens keep their text: through a template and through a `tt`.
macro_rules! a { () => { m!(0x1) } }
macro_rules! tf { ($t:tt) => { m!($t) } }
macro_rules! lit { ($l:literal) => { stringify!($l) } }
fn main() {
    println!("{} {} {} {} {}", m!(0x1), m!(1), m!(01), m!(1_), m!(0b1));
    println!("{} {} {} {}", u!(1u8), u!(1_u8), u!(0x1u8), u!(1));
    println!("{} {} {} {}", c!('a'), c!('\x61'), c!('\u{61}'), c!('\u{0061}'));
    println!("{} {} {} {}", s!("a"), s!("\x61"), s!(r"a"), s!(r#"a"#));
    println!("{} {} {}", r!(r"a"), r!("a"), r!(r#"a"#));
    println!("{} {} {} {}", f!(1.0), f!(1.00), f!(1.0f64), f!(1e0));
    println!("{} {}", b!(b"a"), b!(b"\x61"));
    println!("{} {}", d!(/// a
    ), d!(#[doc = " a"]));
    println!("{} {}", x!(0x1), x!(0x01));
    println!("{} {} {}", a!(), tf!(0x1), tf!(1));
    println!("{} {} {} {}", lit!(0x1), lit!(-0x1), lit!('\x61'), lit!("\x61"));
    println!("{}", stringify!(0x1 1_000u32 0o17 0b1_0 '\u{61}' "a\"b" r#"a"b"#));
}
