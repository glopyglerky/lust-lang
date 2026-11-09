// expect: L0612
// `trim_matches` needs a double-ended searcher: rustc rejects a
// `&str` pattern there (tf-str).
fn main() {
    println!("{}", "ab".trim_matches("a"));
}
