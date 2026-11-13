// expect: L0612
// rustc E0277: core has no `u8: From<i64>` (only lossless conversions).
struct W<T>(T);

impl<T> W<T> {
    fn get(&self) -> u32
    where
        u8: From<i64>,
    {
        8
    }
}

fn main() {
    println!("{}", W(1u8).get());
}
