// expect: L0640
// A BTreeMap key with a written Ord impl: lust std source compares keys
// with < and ==, so a written impl (which may disagree or log) rejects.
use std::collections::BTreeMap;
use std::cmp::Ordering;
#[derive(PartialEq, Eq, Debug)]
struct Rev(u8);
impl PartialOrd for Rev { fn partial_cmp(&self, o: &Rev) -> Option<Ordering> { Some(self.cmp(o)) } }
impl Ord for Rev { fn cmp(&self, o: &Rev) -> Ordering { o.0.cmp(&self.0) } }
fn main() { let mut m = BTreeMap::new(); m.insert(Rev(1), 1); m.insert(Rev(2), 2); println!("{:?}", m); }
