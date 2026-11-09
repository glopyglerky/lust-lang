// lust-reject: L0506
// `memrchr_iter` (an `iter::Rev` over `Memchr`) is not in the adapter
// yet, so the import rejects; the real crate has it.
use memchr::memrchr_iter;

fn main() {
    let h: &[u8] = b"abcabc";
    let v: Vec<usize> = memrchr_iter(b'b', h).collect();
    println!("{:?}", v);
}
