// expect: L0001
// rustc: labels cannot use keyword names, also where `break` names one.
fn main() { loop { break '_; } }
