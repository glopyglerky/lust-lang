// MaybeUninit of a zero-sized type: uninit and zeroed values, on the stack,
// in an Rc, and in an Arc.
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;
#[derive(Debug)]
struct Z;
fn main() {
    let a = MaybeUninit::<Z>::uninit();
    let b: [MaybeUninit<()>; 3] = [MaybeUninit::zeroed(); 3];
    let r = Rc::<Z>::new_uninit();
    let s = Arc::<()>::new_zeroed();
    println!("{:?} {} {:?} {:?}", a, b.len(), r, s);
}
