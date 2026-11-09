// `count` on an exclusive range of more than usize::MAX items panics in
// core's range.rs (Range's `size_hint` expect), not at the caller.
// panic: count overflowed usize
fn main() {
    let n = std::env::args().count() as u128;
    println!("{}", (0u128..10 + n).count());
    println!("{}", (0u128..u128::MAX - n).count());
}
