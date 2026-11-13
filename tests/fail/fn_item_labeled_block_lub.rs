// expect: L0110
// rustc E0308: a labeled block's tail coerces to its first break's fn item
// type in a `let`; two fn items do not join as a fn pointer there.
fn f1() -> i32 {
    1
}
fn f2() -> i32 {
    2
}
fn main() {
    let n = std::env::args().count();
    let v = 'l: {
        if n == 1 {
            break 'l f1;
        }
        f2
    };
    println!("{}", v());
}
