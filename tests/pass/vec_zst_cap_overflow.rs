// A zero-sized element Vec has capacity usize::MAX: a reserve that takes
// its length past that is RawVec's capacity overflow panic.
// panic: capacity overflow
fn main() {
    let mut v: Vec<()> = Vec::new();
    v.push(());
    v.reserve(usize::MAX - 1);
    println!("{} {}", v.len(), v.capacity());
    v.reserve_exact(usize::MAX - 1);
    println!("{}", v.len());
    v.reserve(usize::MAX);
    println!("unreachable");
}
