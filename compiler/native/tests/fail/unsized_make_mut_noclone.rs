// expect: L0112
// rustc E0277: `Rc::<[N]>::make_mut` needs `[N]: CloneToUninit`, which needs `N: Clone`.
use std::rc::Rc;
struct N;
fn main() { let mut a: Rc<[N]> = Rc::from(vec![N]); let m = Rc::make_mut(&mut a); println!("{}", m.len()); }
