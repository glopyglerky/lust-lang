//! Types shared by the separately compiled caller and provider crates.
//! Ordinary Rust layout is intentional except for the explicit C ABI probe.

use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Eq)]
pub struct Aggregate {
    pub tag: u8,
    pub wide: u128,
    pub lanes: [u64; 5],
    pub niche: Option<NonZeroU64>,
    pub text: String,
    pub values: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tagged {
    Empty,
    Number(NonZeroU64),
    Owned { label: String, words: Vec<u64> },
}

#[derive(Debug, PartialEq, Eq)]
pub struct GenericResult<T> {
    pub first: T,
    pub second: T,
    pub count: usize,
}

#[inline(never)]
pub fn generic_fold<T: Copy + Into<u64>, const N: usize>(values: [T; N]) -> GenericResult<u64> {
    GenericResult { first: values.into_iter().map(Into::into).sum(), second: values.into_iter().map(Into::into).product(), count: N }
}

pub trait Transform {
    fn apply(&self, value: u64) -> u64;
    fn exchange(&self, value: Aggregate) -> Aggregate;
}

#[derive(Clone, Default)]
pub struct DropLog(Arc<Mutex<Vec<&'static str>>>);

impl DropLog {
    #[inline(never)]
    pub fn record(&self, event: &'static str) {
        self.0.lock().unwrap().push(event);
    }

    pub fn events(&self) -> Vec<&'static str> {
        self.0.lock().unwrap().clone()
    }
}

/// This is a Rust-defined C calling-convention boundary. No C compiler or
/// external allocator is involved; it does not claim a general C FFI gate.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CRecord {
    pub small: u32,
    pub wide: u64,
    pub pair: [u64; 2],
}

pub type CCallback = extern "C" fn(CRecord) -> CRecord;
