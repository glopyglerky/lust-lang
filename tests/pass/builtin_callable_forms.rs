// The builtins whose C adapter calls a callable argument take any FnMut value rustc takes:
// a closure, a fn item (also a generic one through a generic caller), a fn pointer, and
// any of these behind `&mut`, `&` or a Box. Before tf-build, all but a closure value
// panicked in codegen ("a sort comparator is a closure value").
use std::cmp::Ordering;

fn rev(a: &u32, b: &u32) -> Ordering {
    b.cmp(a)
}
fn key(a: &u32) -> u32 {
    a % 3
}
fn keep(a: &u32) -> bool {
    *a > 1
}
fn bump(a: &mut u32) -> bool {
    *a += 10;
    *a > 11
}
fn probe(a: &u32) -> Ordering {
    a.cmp(&2)
}
fn make() -> String {
    String::from("made")
}
fn by<T, F: FnMut(&T, &T) -> Ordering>(x: &mut [T], cmp: F) {
    x.sort_by(cmp);
}
fn cmp_gen<T: Ord>(a: &T, b: &T) -> Ordering {
    a.cmp(b)
}

fn main() {
    let base = vec![3u32, 1, 2, 2, 5];
    let mut v = base.clone();
    v.sort_by(rev);
    println!("{v:?}");
    let f: fn(&u32, &u32) -> Ordering = rev;
    v.sort_unstable_by(f);
    v.sort_by(cmp_gen);
    println!("{v:?}");
    by(&mut v, rev);
    by(&mut v, cmp_gen::<u32>);
    println!("{v:?}");
    let mut calls = 0;
    let mut counting = |a: &u32, b: &u32| {
        calls += 1;
        a.cmp(b)
    };
    v.sort_by(&mut counting);
    v.sort_unstable_by(&mut counting);
    println!("{v:?} {}", calls > 0);
    v.sort_by(Box::new(rev));
    let k: fn(&u32) -> u32 = key;
    v.sort_by_key(key);
    println!("{v:?}");
    v.sort_unstable_by_key(k);
    v.sort_by_key(&mut |a: &u32| a % 2);
    println!("{v:?}");
    v.sort();
    println!("{:?} {:?}", v.binary_search_by(probe), v.binary_search_by_key(&2, key));
    let p: fn(&u32) -> Ordering = probe;
    println!("{:?} {:?}", v.binary_search_by(p), v.binary_search_by(&mut |a: &u32| a.cmp(&9)));
    let mut w = base.clone();
    w.retain(keep);
    println!("{w:?}");
    let pk: fn(&u32) -> bool = keep;
    w.retain(&pk);
    w.retain_mut(bump);
    println!("{w:?}");
    let mut seen = Vec::new();
    w.retain(&mut |a: &u32| {
        seen.push(*a);
        *a % 2 == 1
    });
    println!("{w:?} {seen:?}");
    println!("{:?} {:?}", (w.len() > 0).then(make), (w.len() > 9).then(make));
    let mk: fn() -> String = make;
    let (s, t) = (String::from("moved"), String::from("kept"));
    println!("{:?} {:?} {:?}", true.then(mk), true.then(move || s), false.then(move || t));
    println!("{:?} {:?}", base.iter().copied().min_by_key(key), base.iter().copied().max_by_key(k));
}
