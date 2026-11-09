// core's `From<NonZero<T>> for T` (ts-rust fast_mode.rs
// `available_parallelism().map_or(4, Into::into)`): `into`, `T::from`, and
// `Into::into` as a value.
use std::num::NonZeroUsize;
use std::sync::OnceLock;
static CORES: OnceLock<usize> = OnceLock::new();
fn main() {
    let n = NonZeroUsize::new(6).unwrap();
    let a: usize = n.into();
    let b = usize::from(n);
    let c = Some(n).map_or(4, Into::into);
    let d: Result<NonZeroUsize, ()> = Ok(n);
    let e = d.map_or(4, Into::into);
    let cores = *CORES.get_or_init(|| std::thread::available_parallelism().map_or(4, Into::into));
    println!("{a} {b} {c} {e} {}", cores > 0);
    let u: u32 = std::num::NonZeroU32::new(3).unwrap().into();
    println!("{u}");
}
