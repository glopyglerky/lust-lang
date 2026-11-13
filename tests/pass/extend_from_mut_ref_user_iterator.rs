// `v.extend(&mut it)` and a generic `IntoIterator` callee with I = &mut UserIter: core's
// `impl Iterator for &mut I` steps the user iterator in place. lust check rejected the direct
// form (L0640) and accepted the generic one, whose instance failed to build.
struct Count(u32);
impl Iterator for Count {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.0 += 1;
        if self.0 < 4 { Some(self.0) } else { None }
    }
}
struct Cnt<'a>(&'a [u32], usize);
impl<'a> Iterator for Cnt<'a> {
    type Item = &'a u32;
    fn next(&mut self) -> Option<&'a u32> {
        let r = self.0.get(self.1);
        self.1 += 1;
        r
    }
}
fn ext<I: IntoIterator>(i: I) -> Vec<I::Item> {
    let mut v = Vec::new();
    v.extend(i);
    v
}
fn main() {
    let mut c = Count(0);
    let mut v = vec![0];
    v.extend(&mut c);
    println!("{:?} {:?}", v, c.next());
    let w = vec![7u32, 8, 9];
    let mut k = Cnt(&w, 0);
    let got = ext(&mut k);
    println!("{:?}", got);
    let mut c = Count(1);
    let s: String = ext(c.by_ref()).iter().map(|x| x.to_string()).collect();
    println!("{} {:?}", s, c.next());
}
