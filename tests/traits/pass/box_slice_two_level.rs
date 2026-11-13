// ARCH 2.4, multi-level clause filter: Box<[T]> has a direct
// IntoIterator impl. The Iterator blanket also matches by header
// through `Iterator for Box<I>`, but that candidate needs
// `[T]: Iterator`, which has no candidate, so it must drop.
// From the round-2 probe select.rs.
fn main() {
    let b: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
    let mut sum = 0;
    for x in b {
        sum += x;
    }
    println!("box slice {sum}");
    let mut s = 0u8;
    for x in vec![4u8, 5] {
        s += x;
    }
    println!("vec {s}");
}
