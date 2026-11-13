// semantics: a matcher compares literal tokens by their written text,
// not their value (rustc's token_name_eq on token::Lit), and forwarded
// tokens keep that text through a template and a `tt`.

macro_rules! one { (1) => { "one" }; ($t:tt) => { "other" }; }
macro_rules! ch { ('a') => { "a" }; ($t:tt) => { "other" }; }
macro_rules! st { ("a") => { "a" }; ($t:tt) => { "other" }; }
macro_rules! doc { (#[doc = r" a"]) => { "doc" }; ($($t:tt)*) => { "other" }; }
macro_rules! fwd { () => { one!(0x1) }; }
macro_rules! via_tt { ($t:tt) => { one!($t) }; }

fn main() {
    println!("{} {} {} {}", one!(1), one!(0x1), one!(01), one!(1_));
    println!("{} {} {}", ch!('a'), ch!('\x61'), ch!('\u{61}'));
    println!("{} {} {}", st!("a"), st!("\x61"), st!(r"a"));
    println!("{} {}", doc!(/// a
    ), doc!(#[doc = " a"]));
    println!("{} {} {}", fwd!(), via_tt!(0x1), via_tt!(1));
    println!("{}", stringify!(0x10 '\x61' "\x62" 1_000u32 r#"z"#));
}
