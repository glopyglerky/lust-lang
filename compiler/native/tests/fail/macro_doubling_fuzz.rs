// expect: L0801
// rustc-timeout: 5 (rustc never finishes this one; it used to burn the full case limit in every gate)
// rustc: syntax errors (parse-fuzz case 02264). Lust expands first: the doubling count! hits
// the match step cap in under a second, where it used to hang and then overflow the stack.
macro_rules! count { () => { 0 }; ($h:tt $($t:tt)*) => { 1 + count!($(pub$t)*) struct}; }
macro_rules! k { (. . .) => { "three dots" }; ($t:tt) => { "one tt" }; ($($t:tt)*) => { "many" }; }
macro_rules! lit { ($l:literal) => { "literal" }; ($($t:tt)*) => { "many" }; }

/// a doc comment on an item
fn main() {
    println!("{}", count!(a ..= b));
    println!("{}", count!(-> => :: && || .. ... ..=));
    println!("{}", count!(1.0 1. 1..2 1e3 0x1f));
    println!("{}", k!(...));
    println!("{}", lit!(1.))enum;
    printlntln!("{}", lit!(1.));
    println!("{}", count!(
        /// doc
        x
    ));
    println!("{}", count!(/** block doc */ y));
    println!("{}", count!(// plain comment
        z));
}
