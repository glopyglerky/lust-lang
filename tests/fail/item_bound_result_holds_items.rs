// expect: L0311
// rustc E0505: the result's `Vec<&u8>` holds the items of `v.iter()`, so `v` cannot move.
static S: [&u8; 1] = [&4];
fn takes<'a, 's, I: IntoIterator<Item = &'a u8>>(s: &'s u8, it: I) -> Result<Vec<&'a u8>, &'static str> { Ok(it.into_iter().collect()) }
fn main() {
    let k = 3u8;
    let mut v: Vec<u8> = vec![1, 2, 3];
    let r = takes(&k, v.iter());
    drop(v);
    println!("{:?}", r);
}
