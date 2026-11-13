use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::LazyLock;

static WORDS: LazyLock<BTreeMap<String, usize>> = LazyLock::new(|| base::counts(&["b", "a", "b"]));

pub struct Meters<T>(pub f64, PhantomData<T>);

pub fn meters<T>(x: f64) -> Meters<T> {
    Meters(x, PhantomData)
}

pub fn words() -> &'static BTreeMap<String, usize> {
    &WORDS
}

/// The first word over `max` letters, through `base::visit` and `?`.
pub fn first_long(words: &[&str], max: usize) -> std::ops::ControlFlow<String, u8> {
    base::visit(words, &mut |w| if w.len() > max { std::ops::ControlFlow::Break(w.to_string()) } else { std::ops::ControlFlow::Continue(()) })?;
    std::ops::ControlFlow::Continue(0)
}
