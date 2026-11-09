// tikv-jemallocator 0.7.0: the Jemalloc type itself (unit struct,
// Copy, Clone, Default, Debug) and its layout.
use tikv_jemallocator::Jemalloc;

fn main() {
    let a = Jemalloc;
    let b = a;
    let c = a.clone();
    let d = Jemalloc::default();
    let e: Jemalloc = Default::default();
    println!("{:?} {:?} {:?} {:?} {:?}", a, b, c, d, e);
    println!("{} {}", std::mem::size_of::<Jemalloc>(), std::mem::align_of::<Jemalloc>());
}
