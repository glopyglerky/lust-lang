// capability: types: raw pointers *const T and *mut T (safe methods only; a deref needs `unsafe`)
// expect: ok

fn main() {
    let mut x = 5u32;
    let p: *const u32 = &x;
    let q: *mut u32 = &mut x;
    let n: *const u8 = std::ptr::null();
    println!("{} {} {}", p.is_null(), q.is_null(), n.is_null());
}
