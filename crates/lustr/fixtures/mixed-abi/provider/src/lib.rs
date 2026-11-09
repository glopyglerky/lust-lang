//! All non-generic boundary functions remain out of line so a mixed build
//! executes both code generators rather than inlining the whole probe.

use mixed_abi_contract::{Aggregate, CCallback, CRecord, DropLog, GenericResult, Tagged, Transform, generic_fold};
use std::cell::Cell;
use std::num::NonZeroU64;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[inline(never)]
pub fn make_aggregate(seed: u64) -> Aggregate {
    Aggregate {
        tag: 7,
        wide: (u128::from(seed) << 80) | 0x1234,
        lanes: [seed, seed + 1, seed + 2, seed + 3, seed + 4],
        niche: NonZeroU64::new(seed),
        text: format!("seed:{seed}"),
        values: vec![seed, seed + 1, seed + 2],
    }
}

#[inline(never)]
pub fn exchange_aggregate(mut value: Aggregate) -> Aggregate {
    value.tag += 1;
    value.wide = value.wide.rotate_left(17);
    value.lanes.reverse();
    value.niche = value.niche.map(|v| NonZeroU64::new(v.get() + 1).unwrap());
    value.text.push_str(":provider");
    value.values.push(89);
    value
}

#[inline(never)]
pub fn through_rust_callback(value: Aggregate, callback: fn(Aggregate) -> Aggregate) -> Aggregate {
    callback(exchange_aggregate(value))
}

#[inline(never)]
pub fn exchange_niches(
    number: Option<NonZeroU64>,
    owned: Option<Box<u64>>,
    borrowed: Option<&u64>,
) -> (Option<NonZeroU64>, Option<Box<u64>>, Option<u64>) {
    (
        number.map(|v| NonZeroU64::new(v.get() + 2).unwrap()),
        owned.map(|mut value| {
            *value += 3;
            value
        }),
        borrowed.copied(),
    )
}

#[inline(never)]
pub fn exchange_enum(value: Tagged) -> Tagged {
    match value {
        Tagged::Empty => Tagged::Empty,
        Tagged::Number(value) => Tagged::Number(NonZeroU64::new(value.get() + 1).unwrap()),
        Tagged::Owned { mut label, mut words } => {
            label.push_str(":provider");
            words.push(words.iter().sum());
            Tagged::Owned { label, words }
        }
    }
}

#[inline(never)]
pub fn generic_instances(callback: fn([u32; 4]) -> GenericResult<u64>) -> (GenericResult<u64>, GenericResult<u64>) {
    (generic_fold([2_u16, 3, 5, 7]), callback([11, 13, 17, 19]))
}

struct ProviderTransform {
    log: DropLog,
    factor: u64,
}

impl Transform for ProviderTransform {
    #[inline(never)]
    fn apply(&self, value: u64) -> u64 {
        self.factor * value
    }

    #[inline(never)]
    fn exchange(&self, value: Aggregate) -> Aggregate {
        exchange_aggregate(value)
    }
}

impl Drop for ProviderTransform {
    #[inline(never)]
    fn drop(&mut self) {
        self.log.record("provider-trait-drop");
    }
}

#[inline(never)]
pub fn make_trait(log: DropLog) -> Box<dyn Transform> {
    Box::new(ProviderTransform { log, factor: 3 })
}

#[inline(never)]
pub fn consume_trait(value: Box<dyn Transform>, aggregate: Aggregate) -> (u64, Aggregate) {
    let applied = value.apply(14);
    let exchanged = value.exchange(aggregate);
    drop(value);
    (applied, exchanged)
}

/// The caller grows allocations created here, then this crate grows and drops
/// allocations that the caller created. Both directions cross the Rust ABI.
#[inline(never)]
pub fn consume_allocations(mut text: String, mut values: Vec<String>) -> (usize, usize) {
    text.reserve(256);
    text.push_str(":provider");
    values.reserve(32);
    values.push(String::from("provider-owned"));
    let lengths = (text.len(), values.iter().map(String::len).sum());
    drop((text, values));
    lengths
}

struct ProviderGuard(DropLog, &'static str);

impl Drop for ProviderGuard {
    #[inline(never)]
    fn drop(&mut self) {
        self.0.record(self.1);
    }
}

#[inline(never)]
pub fn unwind_through_provider(log: DropLog, callback: fn(DropLog)) {
    let _guard = ProviderGuard(log.clone(), "provider-pass-drop");
    callback(log);
}

#[inline(never)]
pub fn panic_in_provider(log: DropLog) {
    let _guard = ProviderGuard(log, "provider-panic-drop");
    panic!("provider-origin");
}

#[inline(never)]
pub fn catch_consumer_panic(log: DropLog, callback: fn(DropLog)) {
    let _guard = ProviderGuard(log.clone(), "provider-catch-drop");
    let outcome = catch_unwind(AssertUnwindSafe(|| callback(log.clone())));
    let payload = outcome.expect_err("a panic must unwind through the consumer frame");
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"provider-origin"));
    log.record("provider-caught");
}

#[inline(never)]
pub fn atomic_roundtrip(value: &AtomicU64, callback: fn(&AtomicU64)) -> u64 {
    value.store(40, Ordering::Release);
    callback(value);
    value.fetch_add(1, Ordering::AcqRel)
}

#[inline(never)]
pub fn increment(value: &AtomicU64) {
    value.fetch_add(1, Ordering::Relaxed);
}

static TLS_DROPS: AtomicUsize = AtomicUsize::new(0);
struct TlsGuard;

impl Drop for TlsGuard {
    fn drop(&mut self) {
        TLS_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

thread_local! {
    static TLS_VALUE: Cell<u64> = const { Cell::new(100) };
    static TLS_GUARD: TlsGuard = const { TlsGuard };
}

#[inline(never)]
pub fn tls_step(delta: u64) -> u64 {
    TLS_GUARD.with(|_| {});
    TLS_VALUE.with(|value| {
        value.set(value.get() + delta);
        value.get()
    })
}

#[inline(never)]
pub fn tls_through_callback(callback: fn(u64) -> u64) -> (u64, u64) {
    (tls_step(3), callback(5))
}

#[inline(never)]
pub fn tls_drops() -> usize {
    TLS_DROPS.load(Ordering::SeqCst)
}

#[inline(never)]
pub extern "C" fn through_c_callback(mut value: CRecord, callback: CCallback) -> CRecord {
    value.small += 1;
    value.wide += 2;
    value.pair.swap(0, 1);
    callback(value)
}
