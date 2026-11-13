// RawVec's capacity overflow at the byte boundary (lust_vec_fits): with_capacity,
// reserve_exact and reserve of isize::MAX / size + 1 elements panic for element sizes 1, 3
// and a large odd size, and keep the Vec as it was; a zero-sized element takes usize::MAX.
// The side just under the boundary would allocate about isize::MAX bytes, so only
// tests/runtime/vecfits_test.c checks it.
use std::panic::catch_unwind;

fn over<T>(name: &str) {
    let size = std::mem::size_of::<T>();
    let n = isize::MAX as usize / size + 1;
    let a = catch_unwind(|| Vec::<T>::with_capacity(n).capacity()).is_err();
    let b = catch_unwind(|| {
        let mut v = Vec::<T>::new();
        v.reserve_exact(n);
        v.capacity()
    })
    .is_err();
    let c = catch_unwind(|| {
        let mut v = Vec::<T>::new();
        v.reserve(n);
        v.capacity()
    })
    .is_err();
    let mut v = Vec::<T>::with_capacity(1);
    let d = catch_unwind(std::panic::AssertUnwindSafe(|| v.reserve_exact(n))).is_err();
    println!("{name}: size {size}, n {n}: {a} {b} {c} {d}, kept {} {}", v.len(), v.capacity());
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    over::<u8>("u8");
    over::<[u8; 3]>("[u8; 3]");
    over::<[u8; 1000003]>("[u8; 1000003]");
    let mut z = Vec::<()>::with_capacity(usize::MAX);
    z.push(());
    let e = catch_unwind(std::panic::AssertUnwindSafe(|| z.reserve(usize::MAX))).is_err();
    println!("(): cap {}, len {}, reserve past usize::MAX panics: {e}", z.capacity(), z.len());
}
