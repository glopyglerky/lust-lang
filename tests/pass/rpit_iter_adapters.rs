// Iterator adapters on a returned `impl Iterator` value (the caller's
// opaque view proves `Iterator` from the bound).
fn evens(n: u32) -> impl Iterator<Item = u32> { (0..n).filter(|x| x % 2 == 0) }
fn main() {
    let s: u32 = evens(9).map(|x| x * 3).sum();
    println!("{} {}", s, evens(5).count());
    let v: Vec<(usize, u32)> = evens(7).enumerate().skip(1).collect();
    println!("{:?}", v);
}
