// A cfg-disabled block-like statement ends at its last block, as in rustc:
// the next statement is kept even when its first token could continue an
// expression (`if`, `{`, `*`, `-`, `(`, `[`, `&`, `'a:`). Each statement
// prints, so a dropped one shows in the output.
#![allow(unused)]
fn t(s: &str) -> i32 {
    println!("{s}");
    1
}
struct P {
    a: i32,
}
impl P {
    fn show(&self) -> i32 {
        t("show")
    }
}
macro_rules! body {
    () => {
        #[cfg(any())]
        if c() { t("x"); }
        if c() { t("macro next"); }
        #[allow(unused)]
        if c() { t("macro kept"); }
        { t("macro block"); }
    };
}
fn c() -> bool {
    true
}
fn main() {
    let mut n = 0;
    let v = [1, 2];
    #[cfg(feature = "nope")]
    if true {
        n += 10;
    }
    if n == 0 {
        n += 100;
    }
    println!("{n}");
    #[cfg(any())]
    if c() { t("x"); } else { t("x"); }
    { t("block"); }
    #[cfg(any())]
    match n { _ => { t("x"); } }
    *(&mut n) += t("deref");
    #[cfg(any())]
    loop { t("x"); break }
    -t("neg");
    #[cfg(any())]
    while false {}
    (t("paren"));
    #[cfg(any())]
    for _ in 0..1 {}
    [t("index")];
    #[cfg(any())]
    if let Some(P { a }) = Some(P { a: 1 }) { t("x"); } else if c() { } else { }
    &t("ref");
    #[cfg(any())]
    'a: { t("x"); break 'a; }
    'b: { t("label"); break 'b; }
    #[cfg(any())]
    if match n { _ => true } { t("x"); }
    (|| t("closure"))();
    #[cfg(any())]
    { P { a: 1 } }.show();
    t("after method");
    #[cfg(any())]
    m_unknown! { anything }
    -t("after brace macro");
    #[cfg(any())]
    { 1 } - t("kept rhs");
    #[cfg(all())]
    'c: { t("labeled kept"); }
    [t("index after label")];
    body!();
    println!("{n}");
}
