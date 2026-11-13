// memchr 2.8.2: size_hint and the collect capacity it sets, without
// next_back or rev (iter_hint has those). Lust rejects size_hint on
// these iterators until std gives user iterators a default; from then
// on this case fails unless the adapter overrides size_hint as
// upstream does.
use memchr::memmem;

fn main() {
    let h: &[u8] = b"banana bandana";
    let mut it = memchr::memchr_iter(b'a', h);
    println!("{:?}", it.size_hint());
    it.next();
    println!("{:?}", it.size_hint());
    println!("{:?}", memchr::memchr3_iter(b'b', b'd', b'n', h).size_hint());
    println!("{:?} {:?}", memmem::find_iter(h, "an").size_hint(), memmem::find_iter(h, "").size_hint());
    let all: Vec<usize> = memmem::find_iter(h, "").collect();
    println!("{} {}", all.len(), all.capacity());
}
