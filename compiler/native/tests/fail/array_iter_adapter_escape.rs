// expect: L0306
// rustc E0597: the same through adapters over an array `IntoIter` and `impl Iterator`,
// with the items pushed in a loop.
fn take<'a>(it: impl Iterator<Item = &'a u32>, out: &mut Vec<&'a u32>) {
    for x in it {
        out.push(x);
    }
}
fn main() {
    let mut v: Vec<&u32> = Vec::new();
    {
        let n = 5u32;
        take([&n, &n].into_iter().zip([1u8, 2]).map(|(a, _)| a).take(1), &mut v);
    }
    println!("{}", v.len());
}
