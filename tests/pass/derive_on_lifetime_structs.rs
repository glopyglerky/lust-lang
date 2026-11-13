// Derives (PartialOrd, Ord, Hash, Default, Debug) on structs with a
// lifetime param: the derived glue and the region engine agree.
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Key<'a> {
    name: &'a str,
    rank: u32,
}

#[derive(Debug, Default)]
struct Opts<'a> {
    label: Option<&'a str>,
    n: u32,
}

fn main() {
    let s = String::from("beta");
    let a = Key { name: "alpha", rank: 2 };
    let b = Key { name: &s, rank: 1 };
    println!("{:?} {:?}", a.cmp(&b), a.partial_cmp(&b) == Some(Ordering::Less));
    let mut v = vec![b, a];
    v.swap(0, 1);
    println!("{:?}", v);
    let o: Opts = Opts::default();
    println!("{:?} {}", o.label, o.n);
    let o2 = Opts { label: Some(&s), ..Default::default() };
    println!("{:?}", o2);
}
