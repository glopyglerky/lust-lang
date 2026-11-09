// memchr 2.8.2 iterator extras: size_hint, next_back, count, and the
// Vec capacity collect picks from size_hint. Lust rejects these calls
// until its Iterator has them; once it does, this case catches an
// adapter that still lacks upstream's overrides.
use memchr::memmem;

fn main() {
    let h: &[u8] = b"banana bandana";
    let mut it = memchr::memchr_iter(b'a', h);
    println!("{:?}", it.size_hint());
    println!("{:?} {:?}", it.next(), it.size_hint());
    println!("{:?} {:?}", it.next_back(), it.size_hint());
    println!("{}", memchr::memchr_iter(b'n', h).count());
    let v: Vec<usize> = memchr::memchr2_iter(b'b', b'd', h).collect();
    println!("{:?} {}", v, v.capacity());
    let f = memmem::find_iter(h, "an");
    println!("{:?}", f.size_hint());
    let e = memmem::find_iter(h, "");
    println!("{:?}", e.size_hint());
    let r: Vec<usize> = memchr::memchr_iter(b'a', h).rev().collect();
    println!("{:?}", r);
    // An empty needle's lower bound is len + 1, which sizes collect.
    let all: Vec<usize> = memmem::find_iter(h, "").collect();
    println!("{} {}", all.len(), all.capacity());
}
