// panic: window size must be non-zero
// `windows(0)` panics as std's `NonZero::new(size).expect(..)` does.
fn main() {
    let v = [1, 2, 3];
    let n = v.len() - 3;
    println!("start");
    let c = v.windows(n).count();
    println!("{c}");
}
