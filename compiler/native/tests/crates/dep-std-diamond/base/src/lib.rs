use std::collections::BTreeMap;
use std::time::Duration;

pub fn step() -> Duration {
    Duration::from_millis(250)
}

pub fn counts(words: &[&str]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for w in words {
        *m.entry(w.to_string()).or_insert(0) += 1;
    }
    m
}

/// Calls `f` on each word and stops at its first `Break` (a generic
/// break type, as the ts-rust visitors use).
pub fn visit<B>(words: &[&str], f: &mut impl FnMut(&str) -> std::ops::ControlFlow<B>) -> std::ops::ControlFlow<B> {
    for w in words {
        f(w)?;
    }
    std::ops::ControlFlow::Continue(())
}
