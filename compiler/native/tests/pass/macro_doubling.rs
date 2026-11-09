// A macro whose recursion doubles its input: 13 levels end with 8192
// tokens. Repetition matching is linear in the matched tokens, so each
// level costs as much as its input, not its square.
macro_rules! dbl {
    ([] $($t:tt)*) => { [$(stringify!($t)),*].len() };
    ([$c:tt $($cs:tt)*] $($t:tt)*) => { dbl!([$($cs)*] $($t)* $($t)*) };
}

fn main() {
    println!("{}", dbl!([a a a a a a a a a a a a a] x));
}
