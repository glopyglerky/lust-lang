// expect: L0104
// `BTreeMap::extract_if` is stable std; lust std source has no `extract_if` yet.
use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<u8, u8> = BTreeMap::new();
    let _ = m.extract_if(.., |_, _| true);
}
