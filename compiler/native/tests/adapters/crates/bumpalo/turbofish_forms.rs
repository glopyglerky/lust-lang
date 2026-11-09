// Upstream's generic parameter order: `alloc_with<F, T>`.
fn main() {
    let b = bumpalo::Bump::new();
    let x = b.alloc_with::<_, i32>(|| 5);
    let y = b.try_alloc_with::<_, u8>(|| 7).unwrap();
    println!("{x} {y}");
}
