use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn build_value() -> u32 {
    let configured: u32 = env!("FIXTURE_BUILD_VALUE").parse().unwrap();
    assert_eq!(configured, GENERATED);
    GENERATED
}

pub async fn deferred_add(left: u32, right: u32) -> u32 {
    std::future::ready(left + right).await
}

struct NoopWake;
impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

pub fn complete<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture future unexpectedly pending"),
    }
}

pub trait LendingRead {
    type Item<'a>
    where
        Self: 'a;
    fn read(&self) -> Self::Item<'_>;
}

pub struct Reader(pub u32);
impl LendingRead for Reader {
    type Item<'a> = &'a u32;
    fn read(&self) -> Self::Item<'_> {
        &self.0
    }
}

/// Reads a value through a raw pointer under its ordinary Rust safety contract.
///
/// # Safety
/// `ptr` must be aligned and point to a live, initialized `u32`.
pub unsafe fn read_raw(ptr: *const u32) -> u32 {
    unsafe { ptr.read() }
}

pub extern "C" fn ffi_increment(value: u32) -> u32 {
    value + 1
}

pub fn array_sum<const N: usize>(values: [u32; N]) -> u32 {
    values.into_iter().sum()
}

/// A controlled, dependency-visible benchmark edit point.
///
/// ```
/// assert_eq!(fixture_core::body_probe(20), 42);
/// ```
pub fn body_probe(value: u32) -> u32 {
    value * 2 + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn features_keep_their_rust_semantics() {
        assert_eq!(build_value(), 7);
        assert_eq!(complete(deferred_add(20, 22)), 42);
        assert_eq!(*Reader(13).read(), 13);
        let value = 19;
        assert_eq!(unsafe { read_raw(&value) }, 19);
        let function: extern "C" fn(u32) -> u32 = ffi_increment;
        assert_eq!(function(20), 21);
        assert_eq!(array_sum([3, 5, 7]), 15);
        assert_eq!(body_probe(20), 42);
    }
}

pub fn benchmark_value() -> u32 { 41 } // BENCH_PROBE
