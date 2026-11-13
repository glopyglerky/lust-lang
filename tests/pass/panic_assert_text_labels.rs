// A loop label is not an operand: the `*` and `-` after `break 'l` are
// unary.
// panic: assertion failed: 'l: { break 'l *p; } == 'm: { break 'm -1; }
fn main() {
    let a = bb(5);
    let p = &a;
    assert!('l: { break 'l *p; } == 'm: { break 'm -1; });
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
