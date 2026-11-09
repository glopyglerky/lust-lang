// Fn items as str patterns (ts-rust scanner_util.rs
// `trim_start_matches(is_white_space_like)`, jsdoc.rs
// `trim_end_matches(char::is_whitespace)`): a plain `fn(char) -> bool`
// runs through the same predicate adapter as a closure, and the char
// predicates parse as eta closures.
fn is_white_space_like(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_line_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn main() {
    let s = "  ab c \n";
    println!("[{}]", s.trim_end_matches(char::is_whitespace));
    println!("[{}]", s.trim_start_matches(is_white_space_like));
    println!("[{}]", s.trim_end_matches(is_line_break));
    println!("{:?}", s.split(char::is_whitespace).collect::<Vec<_>>());
    println!("{:?} {:?}", s.find(is_line_break), s.contains(char::is_alphabetic));
    println!("{:?}", "a1b2".split(char::is_numeric).collect::<Vec<_>>());
    let f = is_white_space_like;
    println!("{:?} {:?}", "x y".split_once(f), "  z".strip_prefix(is_white_space_like));
}
