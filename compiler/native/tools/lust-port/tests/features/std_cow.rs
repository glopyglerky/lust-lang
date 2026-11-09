// feature: std.cow
// form: Cow<'_, str> return that borrows or owns, as in ts-rust normalize_slashes_cow
use std::borrow::Cow;
fn norm(path: &str) -> Cow<'_, str> {
    if path.contains('\\') { Cow::Owned(path.replace('\\', "/")) } else { Cow::Borrowed(path) }
}
fn main() {
    let a = norm("a/b");
    let b = norm("a\\b");
    println!("{} {}", a, b);
    println!("{} {}", matches!(a, Cow::Borrowed(_)), matches!(b, Cow::Owned(_)));
}
