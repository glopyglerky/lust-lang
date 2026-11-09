// `[T; N]: AsRef<[T]>` (and AsMut) as a generic bound, as
// `memmem::Finder::new(b"//")` writes it: the bound is proven for
// an array argument and `as_ref()` inside the generic body gives
// the slice.
fn first<B: AsRef<[u8]>>(b: B) -> Option<u8> {
    b.as_ref().first().copied()
}
fn bump<B: AsMut<[u8]>>(mut b: B) {
    b.as_mut()[0] += 1;
}

fn main() {
    println!("{:?}", first(b"xy"));
    println!("{:?}", first([7u8, 8]));
    println!("{:?}", first(&[9u8, 1]));
    let mut a = [10u8, 20];
    bump(&mut a);
    println!("{:?}", a);
}
