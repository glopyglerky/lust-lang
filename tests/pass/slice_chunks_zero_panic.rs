// panic: chunk size must be non-zero
// `chunks_exact(0)` panics as std's `NonZero::new(size).expect(..)`
// does (tf-slice).
fn main() {
    let v = [1, 2, 3];
    let n = v.len() - 3;
    println!("start");
    let c = v.chunks_exact(n).count();
    println!("{c}");
}
