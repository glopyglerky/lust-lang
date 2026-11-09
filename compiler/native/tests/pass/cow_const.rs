// Cow in const and static items, and generic enum constructors in
// const evaluation generally (their field types substitute the
// instance's args).
use std::borrow::Cow;

struct Method(pub Cow<'static, str>);
impl Method {
    pub const BUILD: Method = Method(Cow::Borrowed("build"));
}

const C: Cow<'static, str> = Cow::Borrowed("k");
static S: Cow<'static, str> = Cow::Borrowed("s");
static BYTES: Cow<'static, [u8]> = Cow::Borrowed(b"xy");
const TABLE: [Cow<'static, str>; 2] = [Cow::Borrowed("a"), Cow::Borrowed("b")];

enum G<T> {
    V(T),
    N,
}
const GV: G<u8> = G::V(1);

const fn pick(b: bool) -> Cow<'static, str> {
    if b { Cow::Borrowed("yes") } else { Cow::Borrowed("no") }
}
const P: Cow<'static, str> = pick(true);

fn main() {
    println!("{} {} {} {:?}", Method::BUILD.0, C, S, BYTES);
    println!("{} {}", TABLE[0], TABLE[1]);
    println!("{} {}", matches!(GV, G::V(1)), P);
    println!("{}", S.len() + BYTES.len());
}
