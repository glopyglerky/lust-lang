// `'static` in a closure's parameter and return annotations (ts-rust's
// `|msg: &'static Message, ..|` shape).
fn main() {
    let name_arg = |name: &'static str| -> Vec<(&'static str, String)> { vec![("name", name.to_string())] };
    println!("{:?}", name_arg("x"));
}
