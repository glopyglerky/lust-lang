// expect: L0110
// rustc E0308: `[u8; N]` is not `[u8; 1]` for every `N`, even if the
// only instance has `N = 1`.
fn f<const N: usize>(a: [u8; N]) -> [u8; 1] {
    a
}

fn main() {
    println!("{:?}", f([7u8]));
}
