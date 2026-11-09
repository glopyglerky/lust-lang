// panic: count overflowed usize
// A range with more than usize::MAX items panics in count, as in rustc.
fn main() {
    println!("{}", (0u64..=u64::MAX).count());
}
