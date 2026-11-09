use std::sync::LazyLock;
mod opts { #[derive(Debug, Clone)] pub enum Value { Nil, Num(i32), Mode(u8) } }
use opts::*;
static MAP: LazyLock<Vec<(&str, Value)>> = LazyLock::new(|| {
    use Value::Mode as V;
    vec![("a", V(1)), ("b", V(2)), ("n", Value::Nil)]
});
fn main() { println!("{:?}", *MAP); }
