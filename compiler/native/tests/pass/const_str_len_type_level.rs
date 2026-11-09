// `str::len` of a literal in consts that array types and const
// generic args read (ts-rust locale tag lengths).
const MAX_ALT_TAGLEN: usize = "en-US-POSIX".len();
const MAX_LEN: usize = MAX_ALT_TAGLEN;
const WIDE: usize = "héllo".len() * 2;
struct T {
    a: [u8; MAX_LEN],
}
fn grandfathered(s: [u8; MAX_ALT_TAGLEN]) -> u8 {
    s[0]
}
struct A<const K: usize>;
impl<const K: usize> A<K> {
    fn k(&self) -> usize {
        K
    }
}
fn main() {
    let t = T { a: [7; MAX_LEN] };
    let w: A<WIDE> = A;
    println!("{} {} {}", grandfathered(t.a), t.a.len(), w.k());
}
