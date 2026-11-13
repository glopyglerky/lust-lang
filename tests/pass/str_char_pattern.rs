// A char as a str search pattern (std's `Pattern` impl for char):
// contains, starts_with, ends_with, strip_prefix, strip_suffix, and
// replace, on str, String, and through a Cow's deref, with ASCII and
// multi-byte chars.
use std::borrow::Cow;

fn norm(path: &str) -> Cow<'_, str> {
    if path.contains('\\') { Cow::Owned(path.replace('\\', "/")) } else { Cow::Borrowed(path) }
}

fn main() {
    let s = "a\\b/c\\d";
    println!("{} {} {}", s.contains('\\'), s.starts_with('a'), s.ends_with('d'));
    println!("{:?} {:?}", s.strip_prefix('a'), s.strip_suffix('x'));
    println!("{}", s.replace('\\', "/"));
    println!("{} {}", norm("a/b"), norm("a\\b"));
    let c: Cow<str> = Cow::Borrowed(s);
    println!("{} {}", c.contains('\\'), c.replace('/', "\\"));
    println!("{}", "héllo".replace('é', "e"));
    let st = String::from("x,y,z");
    println!("{} {:?} {}", st.contains(','), st.strip_prefix('x'), st.starts_with(','));
    let ch = '\\';
    println!("{}", "a\\b".contains(ch));
}
