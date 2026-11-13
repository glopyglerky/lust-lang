// reject: vis matching is greedy: the call's `pub` is the visibility, so the matcher token `pub` after it has nothing to take (ARCH 6.2).
// rustc-error: no rules expected keyword `fn`
macro_rules! m { ($v:vis pub fn $n:ident() -> i32 $b:block) => { $v fn $n() -> i32 $b } }
m!(pub fn f() -> i32 { 7 });
fn main() {}
