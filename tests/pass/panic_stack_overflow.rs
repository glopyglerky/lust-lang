// Deep recursion: Rust's guard-page handler prints its stack overflow
// text and aborts (134); the held partial line is lost. Each frame keeps a
// buffer it reads after the call (opaque to the optimizer), so an optimizing
// build cannot turn the recursion into a loop (rustc -O, lust's release C).
// panic: has overflowed its stack
fn depth(n: u64) -> u64 {
    let pad = std::hint::black_box([n; 16]);
    if n == 0 { 0 } else { 1 + depth(n - 1) + pad[(n % 16) as usize] % 3 }
}
fn main() {
    print!("partial ");
    println!("line");
    print!("held");
    let n = std::env::args().count() as u64;
    println!("{}", depth(n * 100_000_000));
}
