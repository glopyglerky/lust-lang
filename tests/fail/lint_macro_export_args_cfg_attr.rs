// expect: L0524
// rustc invalid_macro_export_arguments (deny), also through cfg_attr.
#[cfg_attr(all(), macro_export(foo))]
macro_rules! m { () => { 1 } }
fn main() { println!("{}", m!()); }
