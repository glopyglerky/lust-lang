// In a macro_rules! body, rustc's transcriber marks every body token
// `Alone` (`- x`), keeps an `$e:expr` fragment's own spacing, puts a
// space after a body's `]`, and the printer parenthesizes the fragment
// where precedence needs it.
// panic: assertion failed: m!(- x+1, [x+1] x+1) || -(x + 1) == 9 || (x + 1).abs() == 99
macro_rules! m {
    ($($t:tt)*) => {
        false
    };
}
macro_rules! check {
    ($a:expr) => {
        assert!(m!(-$a, [$a]$a) || -$a == 9 || $a.abs() == 99)
    };
}
fn main() {
    let x: i32 = bb(1);
    check!(x+1);
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
