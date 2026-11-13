// expect: L0612
// rustc E0277: `&F: FnMut` needs `F: Fn`, and F is only FnMut here.
fn call<G: FnMut(u8) -> u8>(mut g: G) -> u8 { g(1) }
fn wrap<F: FnMut(u8) -> u8>(f: &F) -> u8 { call(f) }
fn main() { println!("{}", wrap(&|x| x)); }
