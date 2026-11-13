// expect: L0120
macro_rules! brk { () => { break 'done 7 } }
fn main() { let _v = 'done: loop { brk!() }; }
