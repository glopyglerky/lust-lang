// Literal forms rustc's lexer accepts at their edges: `_` inside a
// unicode escape, six-digit escapes, high bytes in byte and C strings,
// non-ASCII text in C strings, and a line continuation before tabs.
fn main() {
    let s = ["\u{6_1}", "\u{61_}", "\u{000061}", "\u{1_0_FFFF}", "a\
	  b"];
    for x in s {
        print!("{} ", x.len());
    }
    println!();
    println!("{}", stringify!(c"\x80" c"\u{e9}" c"é" cr#"x"y"# br"\x80" b"\xff" b'\x80'));
    println!("{} {}", b'\x80', b"\xff\n".len());
}
