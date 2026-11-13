// `use std::fmt::Write as _;` brings the trait into scope for `write!`
// into a String; it binds no name, so `io::Write` can keep `Write`,
// and a repeated `as _` import is not a duplicate.
use std::fmt::Write as _;
#[allow(unused_imports)]
use std::fmt::Write as _;
#[allow(unused_imports)]
use std::io::Write;

mod inner {
    use core::fmt::Write as _;

    pub fn tag(s: &mut String, n: u32) {
        let _ = writeln!(s, "[{n:>3}]");
    }
}

fn main() {
    let mut s = String::new();
    let _ = write!(s, "n={}", 3);
    let _ = writeln!(s, "!");
    inner::tag(&mut s, 7);
    print!("{s}");
    println!("len {}", s.len());
}
