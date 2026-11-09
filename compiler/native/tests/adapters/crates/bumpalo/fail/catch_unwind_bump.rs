// expect: L0612
// rustc E0277: upstream `Bump` is not `UnwindSafe`.
fn main() {
    let b = bumpalo::Bump::new();
    let r = std::panic::catch_unwind(move || *b.alloc(5));
    println!("{:?}", r.ok());
}
