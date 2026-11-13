// expect: L0612
// rustc E0277: upstream `dedup_by_key<F, K>` takes the closure type first.
use smallvec::SmallVec;

fn main() {
    let mut v: SmallVec<[i32; 4]> = SmallVec::new();
    v.push(1);
    v.dedup_by_key::<i32, _>(|x| *x);
    println!("{}", v.len());
}
