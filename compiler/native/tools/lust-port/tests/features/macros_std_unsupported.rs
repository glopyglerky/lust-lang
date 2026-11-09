// feature: macros.std_unsupported
// form: write! and writeln! into a String, as in ts-rust emitter output code
use std::fmt::Write as _;
fn main() {
    let mut s = String::new();
    let _ = write!(s, "n={}", 3);
    let _ = writeln!(s, "!");
    println!("{}", s.trim_end().len());
}
