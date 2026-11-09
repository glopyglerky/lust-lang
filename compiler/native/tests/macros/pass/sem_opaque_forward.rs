// semantics: opaque fragment forwarding (ARCH 6.2).
// A matched expr/ty fragment keeps its identity when forwarded to
// another macro. The inner matcher must not re-split its tokens.
// ident and tt stay transparent: the inner matcher can match them
// token by token.

// The review probe (sol-probes/macro_forward.rs): forwarding the
// expr 3 must pick the $x:expr arm (4), not the literal arm (3).
macro_rules! a {
    ($x:expr) => { b!($x) };
}
macro_rules! b {
    (3) => { 3 };
    ($x:expr) => { 4 };
}

// A forwarded ty cannot be matched part by part either: the inner
// macro sees one type, not the tokens Vec < u8 >.
macro_rules! ty_inner {
    (Vec<u8>) => { "split" };
    ($t:ty) => { "whole" };
}
macro_rules! ty_outer {
    ($t:ty) => { ty_inner!($t) };
}

// A forwarded ident CAN match a literal ident arm in the inner macro.
macro_rules! id_inner {
    (special) => { "matched-special" };
    ($i:ident) => { "matched-ident" };
}
macro_rules! id_outer {
    ($i:ident) => { id_inner!($i) };
}

// A forwarded tt also stays transparent.
macro_rules! tt_inner {
    (a b) => { "two" };
    ($($t:tt)*) => { "other" };
}
macro_rules! tt_outer {
    ($($t:tt)*) => { tt_inner!($($t)*) };
}

fn main() {
    println!("direct={} forwarded={}", b!(3), a!(3));
    println!("ty direct={} forwarded={}", ty_inner!(Vec<u8>), ty_outer!(Vec<u8>));
    println!("ident={} {}", id_outer!(special), id_outer!(other));
    println!("tt={}", tt_outer!(a b));
}
