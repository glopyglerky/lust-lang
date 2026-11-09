// feature: modules.imports
// form: file module with mod name; in a sibling file
mod imp_helper;
fn main() { println!("{}", imp_helper::value()); }
