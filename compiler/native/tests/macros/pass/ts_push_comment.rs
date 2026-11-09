// macro: push_comment
// source: crates/ts_goport/src/frontend/parser/jsdoc.rs:365
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
fn collect(parts: &[&str]) -> (Vec<String>, i32, i32) {
    let mut comments: Vec<String> = Vec::new();
    let mut margin: i32 = -1;
    let mut indent: i32 = 0;
    macro_rules! push_comment {
            ($text:expr) => {{
                let text: String = ($text).to_string();
                if margin == -1 {
                    margin = indent;
                }
                indent += text.len() as i32;
                comments.push(text);
            }};
        }
    // ts-rust calls it with String, &str and format! expressions.
    for p in parts { push_comment!(*p); }
    push_comment!(format!("n={}", 3));
    (comments, margin, indent)
}

fn main() {
    let (c, margin, indent) = collect(&["ab", "cde"]);
    println!("comments={c:?} margin={margin} indent={indent}");
}
