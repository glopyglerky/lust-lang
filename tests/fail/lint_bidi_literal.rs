// expect: L0545
// rustc text_direction_codepoint_in_literal (deny): a fn-level allow does not lift it.
#[allow(text_direction_codepoint_in_literal)]
fn main() {
    println!("a⁧b");
}
