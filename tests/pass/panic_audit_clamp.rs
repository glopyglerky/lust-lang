// Integer `clamp` with min > max names both bounds.
// panic: min > max. min = 9, max = 2
fn main() {
    let n = std::env::args().count() as i64;
    println!("{}", (n * 50).clamp(0, 10));
    println!("{}", n.clamp(8 + n, 2));
}
