// expect: L0306
// rustc E0597: an array `IntoIter` passed as `I: Iterator<Item = &'a u32>` yields `&'a u32`
// exactly, so extending the out-param with it needs `&n` to live as long as `v`.
fn take<'a, I: Iterator<Item = &'a u32>>(it: I, out: &mut Vec<&'a u32>) {
    out.extend(it);
}
fn main() {
    let mut v: Vec<&u32> = Vec::new();
    {
        let n = 5u32;
        take([&n].into_iter(), &mut v);
    }
    println!("{}", v.len());
}
