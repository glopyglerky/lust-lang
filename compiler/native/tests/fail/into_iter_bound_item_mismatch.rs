// expect: L0110
// rustc E0271: `&Vec<u8>` iterates `&u8`, not the bound's `u8`.
fn n<I: IntoIterator<Item = u8>>(it: I) -> usize {
    it.into_iter().count()
}

fn main() {
    let v = vec![9u8, 1];
    println!("{}", n(&v));
}
