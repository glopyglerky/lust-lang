// expect: L0001
// rustc: a `meta` fragment parses the expression after `=`, labels included.
macro_rules! m { ($m:meta) => { 1 } }
fn main() { println!("{}", m!(a = 'while: loop { break 1 })); }
