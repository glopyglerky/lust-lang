// Upstream's generic parameter order: `dedup_by_key<F, K>`.
use smallvec::SmallVec;

fn main() {
    let mut v: SmallVec<[i32; 4]> = SmallVec::new();
    v.push(1);
    v.push(1);
    v.push(2);
    v.dedup_by_key::<_, i32>(|x| *x);
    println!("{:?}", v.as_slice());
}
