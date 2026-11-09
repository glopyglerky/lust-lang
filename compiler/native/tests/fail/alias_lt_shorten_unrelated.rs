// expect: L0733
// rustc: lifetime may not live long enough (`R<'a>` returned as `R<'b>`).

type R<'a> = &'a String;
fn shorten<'a, 'b>(x: R<'a>, _y: R<'b>) -> R<'b> { x }
fn main() {}
