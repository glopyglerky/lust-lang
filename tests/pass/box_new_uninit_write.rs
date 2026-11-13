// Box::new_uninit, Box::new_zeroed, and Box::write (std 1.87 safe API):
// drop counts, ZSTs, nested boxes, and a large value.
use std::mem::MaybeUninit;
use std::cell::Cell;
thread_local! { static DROPS: Cell<u32> = Cell::new(0); }
struct D(u32);
impl Drop for D { fn drop(&mut self) { DROPS.with(|d| d.set(d.get() + 1)); println!("drop {}", self.0); } }
#[derive(Clone, Debug)]
struct Big { a: [u64; 64], s: String }
struct Z;
impl Drop for Z { fn drop(&mut self) { println!("drop Z"); } }
fn main() {
    let u: Box<MaybeUninit<D>> = Box::new_uninit();
    drop(u);
    println!("after uninit drop: {}", DROPS.with(|d| d.get()));
    let b = Box::write(Box::new_uninit(), D(7));
    println!("written {}", b.0);
    drop(b);
    println!("drops {}", DROPS.with(|d| d.get()));
    let mut m = Box::<D>::new_uninit();
    m.write(D(9));
    drop(m);
    println!("drops {}", DROPS.with(|d| d.get()));
    let big = Box::write(Box::<Big>::new_uninit(), Big { a: [3; 64], s: "big".to_string() });
    println!("{} {}", big.a[63], big.s);
    let z = Box::write(Box::new_uninit(), Z);
    drop(z);
    let zz: Box<MaybeUninit<Z>> = Box::new_zeroed();
    println!("{:?}", zz);
    let n = Box::write(Box::new_uninit(), Box::write(Box::new_uninit(), 5u8));
    println!("{}", **n + 1);
    let unit = Box::write(Box::new_uninit(), ());
    println!("{:?}", unit);
    let s: Box<String> = Box::write(Box::<String>::new_uninit(), String::from("x"));
    println!("{}", s);
}
