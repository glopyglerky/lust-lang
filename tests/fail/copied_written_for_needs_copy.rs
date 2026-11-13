// expect: L0612
// rustc E0277: `for` over a written `Rev<Copied<slice::Iter<C>>>` needs `C: Copy`.
#[derive(Debug, Clone)]
struct C(u8);
fn f(x: std::iter::Rev<std::iter::Copied<std::slice::Iter<'static, C>>>) {
    for c in x {
        println!("{:?}", c);
    }
}
fn main() {
    println!("ok");
}
