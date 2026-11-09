// expect: L0801
macro_rules! m { () => { m!() } }
fn main() { m!(); }
