// std::cmp::Reverse: reversed Ord and PartialOrd (sort keys, max, NaN), derived Debug, Hash, Default, Eq, and Clone::clone_from.
use std::cmp::Reverse;
use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct F(f64);
fn main() {
    let mut v = vec![3, 1, 4, 1, 5, 9, 2, 6];
    v.sort_by_key(|&x| Reverse(x));
    println!("{:?}", v);
    let mut w = vec![(1, "b"), (2, "a"), (1, "a")];
    w.sort_by_key(|&(n, s)| (Reverse(n), s));
    println!("{:?}", w);
    println!("{:?} {:?}", Reverse(3) < Reverse(4), Reverse(3).cmp(&Reverse(4)));
    println!("{:?} {:#?} {:?}", Reverse("x"), Reverse(1u8), Reverse(Some(2)));
    println!("{:?}", Reverse(F(f64::NAN)).partial_cmp(&Reverse(F(1.0))));
    println!("{} {}", Reverse(F(1.0)) < Reverse(F(f64::NAN)), Reverse(F(2.0)) >= Reverse(F(1.0)));
    let d: Reverse<i32> = Default::default();
    println!("{:?} {}", d, d == Reverse(0));
    let mut s = HashSet::new();
    s.insert(Reverse(5)); s.insert(Reverse(5));
    println!("{}", s.len());
    let mut a = Reverse(String::from("abc")); a.clone_from(&Reverse(String::from("z")));
    println!("{:?} {}", a, a.0);
    let m = [Reverse(1), Reverse(7), Reverse(3)].into_iter().max().unwrap();
    println!("{:?}", m);
}
