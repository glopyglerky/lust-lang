// A generic N of 0 panics at the as_chunks call at run time; a literal 0 is a compile error (R13).
// panic: chunk size must be non-zero
fn f<const N: usize>(s: &[u8]) -> usize { s.as_chunks::<N>().0.len() }
fn main() {
    let s: &[u8] = b"abc";
    println!("{}", f::<2>(s));
    println!("{}", f::<0>(s));
}
