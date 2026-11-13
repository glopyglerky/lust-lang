// feature: derives.unknown
// form: a proc-macro derive from another crate with no lust row of its own
// oracle: lust-only
#[derive(Debug, Builder)]
struct Cfg { name: String }
fn main() { println!("x"); }
