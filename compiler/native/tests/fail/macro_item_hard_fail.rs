// expect: L0806
// rustc: expected an item keyword
macro_rules! m { ($i:item) => { () }; (+) => { () }; }
fn main() { m!(+); }
