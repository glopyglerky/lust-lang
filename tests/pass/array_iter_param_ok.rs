// An array `IntoIter` of borrows passed as a type param iterator, `impl Iterator`, or
// `&mut I`: the items may go into an out-param whose elements the borrowed values outlive,
// or into one that is not used after they die.
fn take<'a, I: Iterator<Item = &'a u32>>(it: I, out: &mut Vec<&'a u32>) {
    out.extend(it);
}
fn push<'a>(it: impl Iterator<Item = &'a u32>, out: &mut Vec<&'a u32>) {
    for x in it {
        out.push(x);
    }
}
fn by_mut<'a, I: Iterator<Item = &'a u32>>(it: &mut I, out: &mut Vec<&'a u32>) {
    *out = it.collect();
}
fn main() {
    let n = 5u32;
    let m = 6u32;
    let mut v: Vec<&u32> = Vec::new();
    {
        take([&n].into_iter(), &mut v);
        push([&m, &n].into_iter().rev().map(|x| x), &mut v);
    }
    println!("{} {}", v.len(), v.iter().map(|x| **x).sum::<u32>());
    let mut w: Vec<&u32> = Vec::new();
    {
        let k = 7u32;
        let mut it = [&k, &n].into_iter();
        by_mut(&mut it, &mut w);
        println!("{} {}", w.len(), w[0]);
    }
}
