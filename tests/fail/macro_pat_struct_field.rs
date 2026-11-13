// expect: L0806
// rustc: expected identifier, found `1`
// A struct pattern field is `name: pat` or a shorthand name.
macro_rules! m { ($p:pat_param | x) => { () }; ($t:tt $e:expr) => { () }; }
fn main() { m!(a { 1 }); }
