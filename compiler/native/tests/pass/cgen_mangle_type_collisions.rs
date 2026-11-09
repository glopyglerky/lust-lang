// Pairs of types whose C type names once collided (one mangled name for
// two types): Option<closure(&u32)> vs Option<&closure(u32)>, the Vec
// forms, Option<(&u32,)> vs Option<&(u32,)>, and Option<&(u8, u8)> vs
// Option<Result<(u8,), (u8,)>>.
fn main() {
    let v = 3u32;
    let a = Some(|| v);
    let c = move || v;
    let b = Some(&c);
    println!("{} {}", a.unwrap()(), b.unwrap()());
    let f = || v;
    let va: Vec<_> = vec![f];
    let vb: Vec<_> = vec![&c];
    println!("{} {}", va[0](), vb[0]());
    let t1 = Some((&v,));
    let t2 = Some(&(7u32,));
    println!("{:?} {:?}", t1, t2);
    let t = (1u8, 2u8);
    let r1: Option<&(u8, u8)> = Some(&t);
    let r2: Option<Result<(u8,), (u8,)>> = Some(Ok((3,)));
    println!("{:?} {:?}", r1, r2);
}
