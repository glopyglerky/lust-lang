// A macro call inside the condition prints its tokens with rustc's token
// printer: source spacing, but no space before `,` and a spaced brace
// group after `!`.
// panic: assertion failed: vec! { bb(1) } == vec![1,2] || m!(a+b,c) || m! {}
macro_rules! m {
    ($($t:tt)*) => {
        false
    };
}
fn main() {
    assert!(vec!{bb(1)} == vec![1 ,2] || m!(a+b ,c) || m!{});
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
