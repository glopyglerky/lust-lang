// feature: modules.path_attr
// form: #[path] module in a sibling file, as in ts-rust diagnostics and fswatch mods
#[path = "path_attr_helper.rs"]
mod helper;
fn main() { println!("{}", helper::value()); }
