// Static init closures whose body compares with `<` and `>` (ts-rust
// VIABLE_KEYWORD_SUGGESTIONS: `.filter(|(text, _)| text.len() > 2)`), and the
// `|| -> R { .. }` form with a generic R and a trailing comma.
use std::collections::HashMap;
use std::sync::LazyLock;
static TEXT_TO_KEYWORD: &[(&str, u8)] = &[("as", 1), ("break", 2), ("const", 3), ("if", 4)];
static VIABLE: LazyLock<Vec<String>> = LazyLock::new(|| {
    TEXT_TO_KEYWORD.iter().filter(|(text, _)| text.len() > 2).map(|(text, _)| (*text).to_string()).collect()
});
static CMP: LazyLock<bool> = LazyLock::new(|| 1 < 2 && 3 > 2);
static TYPED: LazyLock<HashMap<u8, u8>> = LazyLock::new(|| -> HashMap<u8, u8> { [(1, 2)].into_iter().collect() },);
static TYPED2: LazyLock<Vec<u8>> = LazyLock::new(move || -> Vec<u8> { vec![1] });
fn main() {
    println!("{:?} {} {:?} {:?}", *VIABLE, *CMP, TYPED.get(&1), *TYPED2);
}
