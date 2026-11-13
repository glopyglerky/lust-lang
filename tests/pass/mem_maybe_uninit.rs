// `core::mem::MaybeUninit<T>` safe surface (TRIAGE T5):
// uninit, zeroed, new, write, as_ptr, unconditional Copy, and
// Debug as the type name. The unsafe readers stay out (lust
// has no unsafe).
use std::mem::MaybeUninit;
fn main() {
    let mut b = [MaybeUninit::<u8>::uninit(); 3];
    let r0 = b[0].write(65);
    println!("{}", *r0);
    b[1].write(66);
    let mut v: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); 2];
    let r1 = v[0].write(67);
    println!("{}", *r1);
    let z = MaybeUninit::<u32>::zeroed();
    let n = MaybeUninit::new(9u32);
    println!("{:?} {:?} {:?} {}", b[0], z, n, std::mem::size_of::<MaybeUninit<String>>());
    println!("{} {}", b[2].as_ptr().is_null(), v.len());
    let c = n.clone();
    println!("{:?}", c);
}
