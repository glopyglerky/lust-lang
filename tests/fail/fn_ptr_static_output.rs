// expect: L0110
// rustc E0308: `id` returns its argument's lifetime, but the pointer type returns `'static`.
fn id(x: &u8) -> &u8 {
    x
}

fn main() {
    let p: fn(&u8) -> &'static u8 = id;
}
