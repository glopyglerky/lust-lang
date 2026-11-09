// expect: L0524
// rustc invalid_macro_export_arguments (deny).
#[macro_export(foo)]
macro_rules! m { () => { 1 } }
fn main() { println!("{}", m!()); }
