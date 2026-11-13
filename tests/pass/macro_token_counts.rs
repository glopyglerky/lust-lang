macro_rules! count { () => { 0 }; ($h:tt $($t:tt)*) => { 1 + count!($($t)*) }; }
macro_rules! k { (. . .) => { "three dots" }; ($t:tt) => { "one tt" }; ($($t:tt)*) => { "many" }; }
macro_rules! lit { ($l:literal) => { "literal" }; ($($t:tt)*) => { "many" }; }

/// a doc comment on an item
fn main() {
    println!("{}", count!(a ..= b));
    println!("{}", count!(-> => :: && || .. ... ..=));
    println!("{}", count!(1.0 1. 1..2 1e3 0x1f));
    println!("{}", k!(...));
    println!("{}", lit!(1.));
    println!("{}", count!(
        /// doc
        x
    ));
    println!("{}", count!(/** block doc */ y));
    println!("{}", count!(// plain comment
        z));
}
