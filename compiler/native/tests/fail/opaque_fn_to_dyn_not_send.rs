// expect: L0612
// rustc E0277: the hidden type of `mk`'s `impl Fn` holds an `Rc`, so it is not
// `Send` (auto traits leak through `impl Trait`), and the dyn target needs `Send`.
fn mk(n: i32) -> impl Fn() -> i32 {
    let r = std::rc::Rc::new(n);
    move || *r
}
fn main() {
    let b: Box<dyn Fn() -> i32 + Send> = Box::new(mk(1));
    println!("{}", b());
}
