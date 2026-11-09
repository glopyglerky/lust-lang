// expect: L0652
// `Duration` has a niche in std (`Option<Duration>` is 16 bytes); lust
// std source keeps a plain u32, so layout queries on std source types
// reject instead of answering with another size (W-STD-C).
use std::time::Duration;
fn main() {
    println!("{}", std::mem::size_of::<Option<Duration>>());
}
