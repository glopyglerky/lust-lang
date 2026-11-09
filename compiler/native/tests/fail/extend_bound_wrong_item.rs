// expect: L0110
// rustc E0271/E0277: the bound is Extend<u8>, the items are u16.
fn f<M: Extend<u8>>(mut m: M) -> M { m.extend(vec![1u16]); m }
fn main() { let _ = f(Vec::<u8>::new()); }
