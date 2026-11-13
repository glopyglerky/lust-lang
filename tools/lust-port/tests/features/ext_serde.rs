// feature: ext.serde
// oracle: lust-only
// form: serde Deserialize derive with helper attributes, as in ts-rust codegen tools
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonEntry {
    category: String,
    code: u32,
    #[serde(default)]
    reports_unnecessary: bool,
}
fn main() { println!("x"); }
