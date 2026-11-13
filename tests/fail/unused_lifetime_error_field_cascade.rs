// expect: L0102
// rustc rejects only the unknown type. `'c` is used through `P<'c>`; P's own field type does not resolve, so
// lust must not add a follow-on "lifetime parameter `'c` is never used" on R (it did, and
// the ts-rust census charged RuleContext and ExportExtractor to that error).

pub struct P<'c> {
    pub c: &'c mut Missing,
}
pub struct R<'a, 'c> {
    pub tp: &'a mut P<'c>,
}
fn main() {}
