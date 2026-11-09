// expect: L0612
// rustc E0277: upstream `alloc_with<F, T>` takes the closure type first.
fn main() {
    let b = bumpalo::Bump::new();
    let x = b.alloc_with::<i32, _>(|| 5);
    println!("{x}");
}
