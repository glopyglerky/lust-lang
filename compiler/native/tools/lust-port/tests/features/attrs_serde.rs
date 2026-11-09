// feature: attrs.serde
// oracle: lust-only
// form: serde helper attributes under a serde derive, as in ts-rust codegen tools
#[derive(Debug)]
struct Schema {
    #[serde(default)]
    count: i32,
}
fn main() { println!("{}", Schema { count: 0 }.count); }
