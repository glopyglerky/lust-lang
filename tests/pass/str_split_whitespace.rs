// split_whitespace over the exact Unicode whitespace set (see
// unicode_props_hash in tests/pass).
fn main() {
    for w in "a b".split_whitespace() {
        println!("{}", w);
    }
}
