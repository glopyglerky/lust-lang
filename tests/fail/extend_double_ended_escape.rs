// expect: L0306
// rustc E0597: an `I: DoubleEndedIterator<Item = &'a u32>` bound ties the source's items to
// `'a` as an `Iterator` bound does.
fn take<'a, I: DoubleEndedIterator<Item = &'a u32>>(it: I, out: &mut Vec<&'a u32>) {
    out.extend(it);
}
fn main() {
    let mut v: Vec<&u32> = Vec::new();
    {
        let n = [1u32, 2, 3];
        take(n.iter(), &mut v);
    }
    println!("{}", v.len());
}
