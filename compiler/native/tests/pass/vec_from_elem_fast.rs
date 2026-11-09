// std's `vec![x; n]` fast paths and the byte-copy Vec methods. A zero-sized
// element that is `()` or IsZero (`[u8; 0]`, `((), ())`, `[(); 4]`) builds a
// Vec of any length at once, and every method below then runs without a loop
// over the elements. A zero value of an IsZero type takes zeroed memory.
// Each line here finishes at once under rustc (debug); lust must not loop.
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
struct U;

const N: usize = usize::MAX - 1;

fn zst_ops() {
    let a = vec![(); N];
    println!("{} {}", a.len(), a.capacity());
    let b = vec![[0u8; 0]; usize::MAX];
    let c = vec![((), ()); N];
    let d = vec![[(); 4]; N];
    println!("{} {} {}", b.len(), c.len(), d.len());

    let mut v = a.clone();
    v.truncate(N / 2);
    v.extend_from_slice(&a[..N / 2]);
    println!("efs {}", v.len());
    let mut w = vec![(); 1];
    w.append(&mut v);
    println!("append {} {}", w.len(), v.len());
    let t = w.split_off(5);
    println!("split {} {}", w.len(), t.len());
    let s = t[1..].to_vec();
    println!("to_vec {}", s.len());
    let mut h = vec![(); usize::MAX / 2];
    h.extend_from_within(..);
    println!("efw {}", h.len());
    let mut x = s;
    x.insert(1, ());
    x.remove(0);
    x.swap_remove(0);
    println!("insrem {}", x.len());
    let bx = x.into_boxed_slice();
    println!("boxed {}", bx.len());
    let mut it = bx.into_vec().into_iter();
    it.next();
    it.next_back();
    println!("iter {}", it.len());
    let mut y = c;
    y.truncate(7);
    println!("truncate {} {:?}", y.len(), y.last());
}

fn zst_arrays() {
    let a = [U; N];
    println!("{}", a.len());
    let v = a.to_vec();
    let f = Vec::from(a);
    let mut e: Vec<U> = Vec::new();
    e.extend_from_slice(&a);
    let g = e.clone();
    let r: Rc<[U]> = Rc::from(&a[..]);
    println!("{} {} {} {} {}", v.len(), f.len(), e.len(), g.len(), r.len());
    println!("{:?} {:?}", g.first(), &g[N - 2..]);
    // slice `Iter::count` and `vec::IntoIter::{count, last}` are O(1).
    println!("{} {}", a.iter().count(), g.iter().count());
    println!("{} {:?}", g.clone().into_iter().count(), g.into_iter().last());
}

fn is_zero_values() {
    // Zero values of IsZero types, and near misses that are not zero.
    let a = vec![0u8; 5];
    let b = vec![0i64; 3];
    let c: Vec<Option<Box<u32>>> = vec![None; 3];
    let d = vec![(0u8, 0.0f64, '\0', false); 2];
    let e = vec![-0.0f64; 2];
    let f: Vec<Option<bool>> = vec![Some(false); 2];
    let g: Vec<Option<bool>> = vec![None; 2];
    let h: Vec<Option<u32>> = vec![Some(0); 2];
    let i: Vec<Option<u32>> = vec![None; 2];
    let j = vec![[0u16; 3]; 2];
    let k = vec![7u8; 4];
    let l = vec![Some(Box::new(1)); 2];
    let m: Vec<Option<String>> = vec![None; 2];
    let n: Vec<Option<(u8, ())>> = vec![Some((0, ())); 2];
    println!("{a:?} {b:?} {c:?} {d:?} {e:?} {f:?} {g:?}");
    println!("{h:?} {i:?} {j:?} {k:?} {l:?} {m:?} {n:?}");
    let empty = vec![0u8; 0];
    println!("{} {}", empty.len(), empty.capacity());
    // 1 GiB of zeroed memory: calloc, no fill loop.
    let big = vec![0u64; 1 << 27];
    println!("{} {}", big.len(), big[big.len() - 1]);
}

fn main() {
    zst_ops();
    zst_arrays();
    is_zero_values();
}
