// `rposition` over slice and array iterators and Map/Copied/Cloned of
// them (W-STD-B): the index counts down from the length, and the
// predicate sees items from the back.
fn last_slash(s: &str) -> Option<usize> {
    s.as_bytes().iter().rposition(|&c| c == b'/')
}

fn main() {
    let v = vec![1, 2, 3, 2, 1];
    let mut seen = Vec::new();
    let r = v.iter().rposition(|&x| {
        seen.push(x);
        x == 2
    });
    println!("{r:?} {seen:?}");
    println!("{:?} {:?}", last_slash("a/b/c"), last_slash("abc"));
    println!("{:?}", [5u8, 6, 7].into_iter().rposition(|x| x > 5));
    println!("{:?}", v.iter().map(|x| x * 10).rposition(|x| x == 10));
    println!("{:?}", v.iter().copied().rposition(|x| x > 9));
    let mut w = vec![4, 5, 6];
    println!("{:?} {w:?}", w.iter_mut().rposition(|x| {
        *x += 1;
        *x == 6
    }));
    let mut it = v.iter();
    println!("{:?} {:?}", it.rposition(|&x| x == 3), it.next());
}
