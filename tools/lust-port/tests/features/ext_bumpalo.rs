// feature: ext.bumpalo
// form: a leaked Bump with an allocation limit and try_alloc_with, as leak_arena.rs
// cargo-dep: bumpalo = "=3.20.3"
use bumpalo::Bump;
fn main() {
    let bump: &'static Bump = Box::leak(Box::new(Bump::with_capacity(256)));
    bump.set_allocation_limit(Some(bump.allocated_bytes()));
    let x: &'static mut u64 = bump.alloc(7);
    *x += 1;
    let s = bump.alloc_slice_copy(&[1u8, 2, 3]);
    println!("{} {:?} {}", x, s, bump.try_alloc_with(|| [0u8; 4096]).is_err());
}
