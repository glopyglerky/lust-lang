// expect: L0545
// rustc text_direction_codepoint_in_comment (deny).
fn main() {
    // a ‮ b
    println!("ok");
}
