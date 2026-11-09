// Written projections, bounds and adapter types through the std adapters whose
// Iterator impls have where clauses (Cloned, Copied, Chain, Rev, Zip, Flatten), with
// the impls they need (fx-projwf).
use std::iter::{Chain, Cloned, Copied, Enumerate, Flatten, Peekable, Rev, Zip};
use std::slice;

#[derive(Debug, Clone)]
struct C(u8);
#[derive(Debug, Clone, Copy)]
struct K(u8);

type CItem = <Cloned<slice::Iter<'static, C>> as Iterator>::Item;

struct Holder {
    k: <Copied<slice::Iter<'static, K>> as Iterator>::Item,
}

fn cloned_item(c: <Cloned<slice::Iter<'static, C>> as Iterator>::Item) -> u8 {
    c.0
}

fn zip_item(z: <Zip<Cloned<slice::Iter<'static, C>>, Copied<slice::Iter<'static, u8>>> as Iterator>::Item) -> u8 {
    z.0.0 + z.1
}

fn needs() -> u8
where
    Rev<Cloned<slice::Iter<'static, C>>>: Iterator<Item = C>,
    Chain<Copied<slice::Iter<'static, K>>, Copied<slice::Iter<'static, K>>>: Iterator,
{
    3
}

fn generic_item<'a, T: Clone>(_v: &'a [T], x: <Cloned<slice::Iter<'a, T>> as Iterator>::Item) -> T {
    x
}

fn count_all(x: Peekable<Cloned<slice::Iter<'_, C>>>) -> usize {
    x.count()
}

fn sum_flat(x: Cloned<Flatten<slice::Iter<'_, Vec<K>>>>) -> u32 {
    let mut s = 0;
    for k in x {
        s += k.0 as u32;
    }
    s
}

fn enumerated(mut x: Enumerate<Copied<slice::Iter<'_, K>>>) -> Option<(usize, K)> {
    x.next()
}

fn generic_count<T: Copy>(x: Rev<Copied<slice::Iter<'_, T>>>) -> usize {
    x.count()
}

fn main() {
    let c: CItem = C(4);
    let h = Holder { k: K(5) };
    println!("{} {} {}", cloned_item(c), h.k.0, zip_item((C(1), 2)));
    println!("{}", needs());
    let v = vec![C(7), C(8)];
    println!("{:?}", generic_item(&v, C(9)));
    println!("{}", count_all(v.iter().cloned().peekable()));
    let vv = vec![vec![K(1), K(2)], vec![K(3)]];
    println!("{}", sum_flat(vv.iter().flatten().cloned()));
    let ks = [K(6), K(7)];
    println!("{:?}", enumerated(ks.iter().copied().enumerate()));
    println!("{}", generic_count(ks.iter().copied().rev()));
}
