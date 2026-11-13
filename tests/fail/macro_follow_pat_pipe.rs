// expect: L0807
macro_rules! m { ($p:pat |) => { 0 } }
fn main() {}
