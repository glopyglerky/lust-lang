// `debug_assert!` with no message reads like `assert!`.
// panic: assertion failed: k * 2 == 3 - k + 1
fn main() {
    let k = std::env::args().count() as i64;
    debug_assert!(k > 0);
    println!("{}", k);
    debug_assert!(k*2==3-k+1);
}
