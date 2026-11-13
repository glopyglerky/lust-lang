// A bound that writes `&'static Message` ties the closure argument to
// `'static`, so a `|m: &'static Message|` closure fits it, passed
// straight or from a `let` (rustc accepts).
struct Message { text: &'static str }
static M: Message = Message { text: "hi" };
fn report<F: FnMut(&'static Message, i32)>(mut f: F) { f(&M, 3); }
fn main() {
    let mut keep: Vec<&'static Message> = Vec::new();
    report(|m: &'static Message, n: i32| { if n > 0 { keep.push(m); } });
    println!("{} {}", keep.len(), keep[0].text);
    more();
}
fn named<'a, F: FnMut(&'a i32)>(mut f: F, z: &'a i32) {
    f(z);
}
fn more() {
    let z = 3;
    let mut out: Vec<&i32> = Vec::new();
    let g = |x| out.push(x);
    named(g, &z);
    let mut keep: Vec<&'static Message> = Vec::new();
    let h = |m: &'static Message, n: i32| {
        if n > 0 {
            keep.push(m);
        }
    };
    report(h);
    println!("{} {}", out.len(), keep.len());
}
