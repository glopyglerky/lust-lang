// PhantomData<T> Debug prints `PhantomData<{type_name::<T>()}>` and Hash
// writes nothing, with no bound on T (core 1.99), in fields, containers,
// generic instances, and pretty or padded formats.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
#[derive(Debug, Hash, Clone, PartialEq)]
struct G<T> { p: PhantomData<T>, n: u8 }
#[derive(Debug, Hash)]
struct H { p: PhantomData<NoTraits>, q: Vec<PhantomData<String>> }
struct NoTraits;
mod inner { #[derive(Debug, Hash)] pub struct Deep<T>(pub T); }
fn main() {
    let p: PhantomData<u8> = PhantomData;
    println!("{:?} [{:>30?}] {:#?}", p, p, p);
    println!("{:?}", vec![PhantomData::<String>, PhantomData]);
    println!("{:?}", Some(PhantomData::<&[u8]>));
    let g: G<u16> = G { p: PhantomData, n: 2 };
    println!("{:?}", g);
    println!("{:#?}", G::<inner::Deep<Option<i64>>> { p: PhantomData, n: 3 });
    let h0 = H { p: PhantomData, q: vec![PhantomData] };
    println!("{:?}", h0);
    let mut h = DefaultHasher::new();
    g.hash(&mut h);
    h0.hash(&mut h);
    PhantomData::<NoTraits>.hash(&mut h);
    println!("{}", h.finish());
    println!("{}", format!("{:?}", PhantomData::<(char, f64, [u16; 3])>));
}
