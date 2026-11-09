// reject: a `?` transcription repetition takes no separator, checked at definition time even when the macro is never called (ARCH 6.2). Lust code: L0807.
// rustc-error: macro repetition operator does not take a separator
macro_rules! m { ($($i:ident),*) => { $($i),? } }
fn main() {}
