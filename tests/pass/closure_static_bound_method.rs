// Methods with a bound that writes `&'static Message`, inherent and
// trait: a closure written into the call, and a `let` closure, take the
// argument as `'static` and nothing else of the call (rustc accepts).
struct Message {
    text: &'static str,
}
static M: Message = Message { text: "hi" };
struct S;
impl S {
    fn report<F: FnMut(&'static Message)>(&self, mut f: F) {
        f(&M);
    }
}
trait Rep {
    fn each<F: FnMut(&'static Message)>(&self, f: F);
}
impl Rep for S {
    fn each<F: FnMut(&'static Message)>(&self, mut f: F) {
        f(&M);
    }
}
fn main() {
    let mut keep: Vec<&Message> = Vec::new();
    S.report(|m| keep.push(m));
    let s = S;
    s.each(|m| keep.push(m));
    let add = |m| keep.push(m);
    s.report(add);
    let mut n = 0;
    s.report(|m| {
        let _ = m;
        n += 1;
    });
    println!("{} {} {}", keep.len(), keep[0].text, n);
}
