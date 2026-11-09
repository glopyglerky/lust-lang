// expect: L0612
// rustc E0271: the hidden iterator yields u8, the bound says u32.
fn bad() -> impl Iterator<Item = u32> {
    vec![2u8, 4].into_iter()
}

fn main() {}
