// Box::new_uninit_slice/new_zeroed_slice and Rc/Arc::new_uninit/new_zeroed (std safe API).
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;
struct D(u8);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let mut b = Box::<[u32]>::new_uninit_slice(4);
    for (i, s) in b.iter_mut().enumerate() { s.write(i as u32 * 10); }
    println!("{} {:?}", b.len(), b[1]);
    let z: Box<[MaybeUninit<D>]> = Box::new_zeroed_slice(3);
    println!("{}", z.len());
    drop(z);
    let e = Box::<[String]>::new_uninit_slice(0);
    println!("{:?}", e);
    let u = Box::<[()]>::new_uninit_slice(1000);
    println!("{}", u.len());
    let mut r = Rc::<D>::new_uninit();
    Rc::get_mut(&mut r).unwrap().write(D(1));
    println!("{:?} {}", r, Rc::strong_count(&r));
    let a: Arc<MaybeUninit<D>> = Arc::new_zeroed();
    let a2 = a.clone();
    println!("{:?} {}", a2, Arc::strong_count(&a));
    let big = Box::<[u64; 512]>::new_uninit();
    let big = Box::write(big, [7; 512]);
    println!("{}", big.iter().sum::<u64>());
}
