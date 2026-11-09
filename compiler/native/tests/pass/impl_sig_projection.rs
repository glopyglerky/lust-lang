// Projections in impl method signatures (`-> Option<Self::Item>`,
// `-> Self::Out`, `-> Self::Hasher`) and a `<M as Make>::Out` parameter,
// with a user BuildHasher and Iterator (W-STD-B coverage).
struct Owner(String, usize);
struct It<'a>(&'a Owner);
impl<'a> Iterator for It<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let o: &'a Owner = self.0;
        if o.1 > 0 { Some(o.0.as_str()) } else { None }
    }
}
trait Make { type Out; fn make(&self) -> Self::Out; fn pair(&self) -> (Self::Out, Self::Out); }
struct M(u8);
impl Make for M {
    type Out = Vec<u8>;
    fn make(&self) -> Self::Out { vec![self.0] }
    fn pair(&self) -> (Self::Out, Self::Out) { (self.make(), vec![]) }
}
fn take(x: <M as Make>::Out) -> usize { x.len() }
#[derive(Clone)]
struct Fx {
    hash: u64,
}
impl std::hash::Hasher for Fx {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.hash = (self.hash ^ b as u64).wrapping_mul(0x100000001b3);
        }
    }
    fn finish(&self) -> u64 {
        self.hash
    }
}
#[derive(Clone, Copy, Default)]
struct FxBuild;
impl std::hash::BuildHasher for FxBuild {
    type Hasher = Fx;
    fn build_hasher(&self) -> Self::Hasher {
        Fx { hash: 7 }
    }
}
fn main() {
    let mut hm: std::collections::HashMap<u32, i32, FxBuild> = std::collections::HashMap::with_hasher(FxBuild);
    hm.insert(7, 2);
    println!("{} {:?}", hm.len(), hm.get(&7));
    let o = Owner(String::from("hi"), 1);
    let mut it = It(&o);
    println!("{:?}", it.next());
    let m = M(4);
    println!("{:?} {:?} {}", m.make(), m.pair(), take(vec![1, 2]));
}
