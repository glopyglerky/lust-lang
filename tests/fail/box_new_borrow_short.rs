// expect: L0306
// rustc E0597: `Box::new(&k)` holds the borrow of `k`.
fn main() {
    let b;
    {
        let k = 3u8;
        b = Box::new(&k);
    }
    println!("{}", b);
}
