// An allocation failure aborts with std's alloc error hook text: the size
// line, then the RUST_BACKTRACE note (hunt7 05:15). black_box keeps the
// allocation: an optimizer may drop an unused one (rustc -O does), and then
// nothing fails.
fn main() {
    println!("start");
    let b = std::hint::black_box(Box::<[u8]>::new_uninit_slice(usize::MAX / 2));
    println!("len={}", b.len());
}
