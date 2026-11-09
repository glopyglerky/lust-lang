// expect: L0505
// rustc E0433: lust's hidden collections module is not a user path.
fn main() {
    let m: __lust_alloc_collections::BTreeMap<u8, u8> = __lust_alloc_collections::BTreeMap::new();
}
