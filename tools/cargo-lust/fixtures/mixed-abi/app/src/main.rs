use mixed_abi_contract::{Aggregate, CRecord, DropLog, GenericResult, Tagged, Transform, generic_fold};
use mixed_abi_provider as provider;
use std::cell::Cell;
use std::num::NonZeroU64;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

const REPORT: &str = "mixed-abi passed: aggregates=3 generics=2 traits=2 niches=6 enums=3 allocation-directions=2 unwind-directions=2 atomics=8000 tls-threads=2 c-callbacks=1";

#[inline(never)]
fn consumer_callback(mut value: Aggregate) -> Aggregate {
    value.tag += 2;
    value.text.push_str(":consumer");
    value.values.push(97);
    value
}

#[inline(never)]
fn consumer_generic(values: [u32; 4]) -> GenericResult<u64> {
    generic_fold(values)
}

struct ConsumerTransform(DropLog);

impl Transform for ConsumerTransform {
    #[inline(never)]
    fn apply(&self, value: u64) -> u64 {
        value + 28
    }

    #[inline(never)]
    fn exchange(&self, value: Aggregate) -> Aggregate {
        consumer_callback(value)
    }
}

impl Drop for ConsumerTransform {
    #[inline(never)]
    fn drop(&mut self) {
        self.0.record("consumer-trait-drop");
    }
}

struct ConsumerGuard(DropLog, &'static str);

impl Drop for ConsumerGuard {
    #[inline(never)]
    fn drop(&mut self) {
        self.0.record(self.1);
    }
}

#[inline(never)]
fn panic_in_consumer(log: DropLog) {
    let _guard = ConsumerGuard(log, "consumer-panic-drop");
    panic!("consumer-origin");
}

#[inline(never)]
fn unwind_through_consumer(log: DropLog) {
    let _guard = ConsumerGuard(log.clone(), "consumer-pass-drop");
    provider::panic_in_provider(log);
}

#[inline(never)]
fn atomic_callback(value: &AtomicU64) {
    assert_eq!(value.compare_exchange(40, 41, Ordering::AcqRel, Ordering::Acquire), Ok(40));
}

#[inline(never)]
fn consumer_increment(value: &AtomicU64) {
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
    static TLS_VALUE: Cell<u64> = const { Cell::new(200) };
    static TLS_GUARD: TlsGuard = const { TlsGuard };
}

#[inline(never)]
fn consumer_tls_step(delta: u64) -> u64 {
    TLS_GUARD.with(|_| {});
    TLS_VALUE.with(|value| {
        value.set(value.get() + delta);
        value.get()
    })
}

#[inline(never)]
extern "C" fn consumer_c_callback(mut value: CRecord) -> CRecord {
    value.small *= 2;
    value.wide *= 3;
    value.pair[1] += 5;
    value
}

fn qualify() {
    assert!(cfg!(panic = "unwind"), "this contract requires genuine unwind semantics");

    let value = provider::make_aggregate(11);
    assert_eq!(value.tag, 7);
    assert_eq!(value.wide, (11_u128 << 80) | 0x1234);
    assert_eq!(value.lanes, [11, 12, 13, 14, 15]);
    assert_eq!(value.niche.map(NonZeroU64::get), Some(11));
    assert_eq!(value.text, "seed:11");
    assert_eq!(value.values, [11, 12, 13]);
    let exchanged = provider::through_rust_callback(value, consumer_callback);
    assert_eq!(exchanged.tag, 10);
    assert_eq!(exchanged.wide, ((11_u128 << 80) | 0x1234).rotate_left(17));
    assert_eq!(exchanged.lanes, [15, 14, 13, 12, 11]);
    assert_eq!(exchanged.niche.map(NonZeroU64::get), Some(12));
    assert_eq!(exchanged.text, "seed:11:provider:consumer");
    assert_eq!(exchanged.values, [11, 12, 13, 89, 97]);
    // Grow provider allocations here, forcing cross-crate reallocation, then
    // let the consumer drop the provider-created aggregate.
    let mut exchanged = exchanged;
    exchanged.text.reserve(1024);
    exchanged.values.reserve(1024);
    exchanged.text.push_str(":grown");
    exchanged.values.push(101);
    assert!(exchanged.text.ends_with(":grown"));
    assert_eq!(exchanged.values.last(), Some(&101));
    drop(exchanged);
    assert_eq!(provider::consume_allocations(String::from("consumer"), vec![String::from("one"), String::from("four")]), (17, 21));

    let borrowed = 19;
    let (number, owned, reference) = provider::exchange_niches(NonZeroU64::new(7), Some(Box::new(13)), Some(&borrowed));
    assert_eq!(number.map(NonZeroU64::get), Some(9));
    assert_eq!(owned.as_deref(), Some(&16));
    assert_eq!(reference, Some(19));
    drop(owned);
    assert_eq!(provider::exchange_niches(None, None, None), (None, None, None));
    assert_eq!(provider::exchange_enum(Tagged::Empty), Tagged::Empty);
    assert_eq!(provider::exchange_enum(Tagged::Number(NonZeroU64::new(23).unwrap())), Tagged::Number(NonZeroU64::new(24).unwrap()));
    assert_eq!(
        provider::exchange_enum(Tagged::Owned { label: String::from("consumer"), words: vec![2, 3, 5] }),
        Tagged::Owned { label: String::from("consumer:provider"), words: vec![2, 3, 5, 10] }
    );

    let (provider_generic, consumer_generic) = provider::generic_instances(consumer_generic);
    assert_eq!(provider_generic, GenericResult { first: 17, second: 210, count: 4 });
    assert_eq!(consumer_generic, GenericResult { first: 60, second: 46189, count: 4 });

    let trait_drops = DropLog::default();
    let provider_trait = provider::make_trait(trait_drops.clone());
    assert_eq!(provider_trait.apply(14), 42);
    let trait_value = provider_trait.exchange(provider::make_aggregate(31));
    assert_eq!(trait_value.text, "seed:31:provider");
    assert_eq!(trait_value.tag, 8);
    drop((trait_value, provider_trait));
    let consumer_trait: Box<dyn Transform> = Box::new(ConsumerTransform(trait_drops.clone()));
    let (applied, trait_value) = provider::consume_trait(consumer_trait, provider::make_aggregate(37));
    assert_eq!(applied, 42);
    assert_eq!(trait_value.text, "seed:37:consumer");
    assert_eq!(trait_value.tag, 9);
    drop(trait_value);
    assert_eq!(trait_drops.events(), ["provider-trait-drop", "consumer-trait-drop"]);

    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info| {
        let message = info.payload().downcast_ref::<&str>().copied().unwrap_or("unexpected payload");
        eprintln!("mixed-abi panic hook: {message}");
    }));
    let consumer_unwind = DropLog::default();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let _guard = ConsumerGuard(consumer_unwind.clone(), "consumer-catch-scope-drop");
        provider::unwind_through_provider(consumer_unwind.clone(), panic_in_consumer);
    }));
    let payload = outcome.expect_err("consumer panic must cross provider's cleanup frame");
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"consumer-origin"));
    assert_eq!(consumer_unwind.events(), ["consumer-panic-drop", "provider-pass-drop", "consumer-catch-scope-drop"]);
    let provider_unwind = DropLog::default();
    provider::catch_consumer_panic(provider_unwind.clone(), unwind_through_consumer);
    assert_eq!(provider_unwind.events(), ["provider-panic-drop", "consumer-pass-drop", "provider-caught", "provider-catch-drop"]);
    std::panic::set_hook(old_hook);

    let value = AtomicU64::new(0);
    assert_eq!(provider::atomic_roundtrip(&value, atomic_callback), 41);
    assert_eq!(value.load(Ordering::Acquire), 42);
    let total = Arc::new(AtomicU64::new(0));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let total = Arc::clone(&total);
            std::thread::spawn(move || {
                for _ in 0..1000 {
                    provider::increment(&total);
                    consumer_increment(&total);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(total.load(Ordering::SeqCst), 8000);

    assert_eq!(provider::tls_drops(), 0);
    assert_eq!(TLS_DROPS.load(Ordering::SeqCst), 0);
    for _ in 0..2 {
        std::thread::spawn(|| {
            assert_eq!(provider::tls_step(2), 102);
            assert_eq!(provider::tls_through_callback(consumer_tls_step), (105, 205));
            assert_eq!(consumer_tls_step(7), 212);
        })
        .join()
        .unwrap();
    }
    assert_eq!(provider::tls_drops(), 2);
    assert_eq!(TLS_DROPS.load(Ordering::SeqCst), 2);

    let value = CRecord { small: 5, wide: 11, pair: [13, 17] };
    assert_eq!(provider::through_c_callback(value, consumer_c_callback), CRecord { small: 12, wide: 39, pair: [17, 18] });
}

fn main() {
    qualify();
    println!("{REPORT}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn bidirectional_abi_contract() {
        super::qualify();
    }
}
