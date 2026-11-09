// expect: L0809
// rustc prints an `$x:ident` substitution as an always-spaced name
// (`x (1)`) and an `$x:tt` one with its source spacing (`x(1)`); lust's
// token trees do not keep which kind it was, so it rejects the text.
macro_rules! m {
    ($($t:tt)*) => {
        false
    };
}
macro_rules! t {
    ($a:ident) => {
        assert!(!m!($a(1)))
    };
}
fn main() {
    let x = 1;
    let _ = x;
    t!(x);
    println!("ok");
}
