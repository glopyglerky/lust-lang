// Box::new_uninit_slice past isize::MAX bytes: RawVec's capacity overflow
// panic, caught; the box is never written (this crashed under unwind).
use std::panic;
fn main() {
    panic::set_hook(Box::new(|_| {}));
    let r = panic::catch_unwind(|| Box::<[u64]>::new_uninit_slice(usize::MAX / 4).len());
    match r {
        Ok(n) => println!("ok {}", n),
        Err(e) => println!("err {}", e.downcast_ref::<&str>().copied().or(e.downcast_ref::<String>().map(|s| s.as_str())).unwrap_or("?")),
    }
    let z = Box::<[()]>::new_uninit_slice(usize::MAX);
    println!("{}", z.len());
}
