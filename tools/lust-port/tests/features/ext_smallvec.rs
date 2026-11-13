// feature: ext.smallvec
// form: SmallVec<[T; N]> push past the inline capacity, as checker TypeSet
// cargo-dep: smallvec = "=1.15.1"
use smallvec::SmallVec;
fn main() {
    let mut v: SmallVec<[u32; 4]> = SmallVec::new();
    for i in 0..6 {
        v.push(i);
        println!("{} {} {}", v.len(), v.capacity(), v.spilled());
    }
    println!("{:?}", v.as_slice());
}
