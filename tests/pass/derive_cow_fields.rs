// Derives over `Cow` fields (W-BENCH2 bug 4, the Cow half fx-cow
// extends with `Default`): Clone, PartialEq, and Debug compare
// and clone through the borrowed value.
use std::borrow::Cow;
#[derive(Clone, PartialEq, Debug)]
struct W {
    c: Cow<'static, str>,
}
fn main() {
    let a = W { c: Cow::Borrowed("x") };
    let b = a.clone();
    let owned = W { c: Cow::Owned("x".to_string()) };
    println!("{} {} {:?}", a == b, a == owned, b);
}
