// panic: index out of bounds: the len is 0 but the index is 0
// Indexing a zero-length array temporary is bounds checked: cgen gives a
// zero-sized promoted temp no storage (its address is its alignment), and
// that shortcut must still run the place's checks.
fn main() {
    let i = std::hint::black_box(0usize);
    println!("{:?} {:?}", [7u8; 0].get(i), [(); 3][i]);
    println!("{}", [7u8; 0][i]);
}
