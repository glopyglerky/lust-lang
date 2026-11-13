// `sort`, `sort_unstable`, and `sort_by_key` compare with
// `PartialOrd::lt`, not `Ord::cmp`: a hand-written `PartialOrd` that
// disagrees with the derived `Ord` decides the order. `max`, `min`,
// and `binary_search` use `Ord::cmp` and keep the derived order.
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
struct K(i32);

impl PartialOrd for K {
    fn partial_cmp(&self, o: &K) -> Option<Ordering> {
        Some(o.0.cmp(&self.0))
    }
}

// A non-generic derived wrapper: its derived `partial_cmp` is
// `Some(self.cmp(other))`, so it orders by `K`'s `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct W(K);

// A generic derived wrapper: its derived `partial_cmp` compares
// fields with `partial_cmp`, so it orders by `K`'s written impl.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct G<T>(T);

fn sorted<T: Ord + Clone>(mut v: Vec<T>) -> Vec<T> {
    v.sort();
    v
}

fn main() {
    let mut v = vec![K(1), K(3), K(2)];
    v.sort();
    println!("{:?}", v);
    let mut v = vec![K(1), K(3), K(2), K(3)];
    v.sort_unstable();
    println!("{:?}", v);
    let mut v = vec![(K(2), "b"), (K(1), "a"), (K(3), "c")];
    v.sort_by_key(|p| p.0);
    println!("{:?}", v);
    let mut v = vec![(K(1), 9), (K(3), 8), (K(2), 7)];
    v.sort();
    println!("{:?}", v);
    println!("{:?}", sorted(vec![K(1), K(3), K(2)]));
    let mut v = vec![W(K(1)), W(K(3)), W(K(2))];
    v.sort();
    println!("{:?}", v);
    let mut v = vec![G(K(1)), G(K(3)), G(K(2))];
    v.sort();
    println!("{:?}", v);
    let mut v = vec![Some(K(2)), None, Some(K(1))];
    v.sort();
    println!("{:?}", v);
    let v = vec![K(3), K(2), K(1)];
    println!("{:?} {:?} {:?}", v.iter().max(), v.iter().min(), v.binary_search(&K(2)));
    println!("{:?}", v.binary_search_by_key(&&2, |k| &k.0));
    // `Cow` sorts as the value it derefs to, in both orders.
    let mut cows: Vec<std::borrow::Cow<'static, str>> = vec![std::borrow::Cow::Borrowed("b"), std::borrow::Cow::Owned(String::from("a"))];
    cows.sort();
    println!("{:?}", cows);
    // A key may borrow from a capture, just not from the element.
    let keys = vec![30, 10, 20];
    let mut idx = vec![2usize, 0, 1];
    idx.sort_by_key(|x| &keys[*x]);
    println!("{:?}", idx);
}
