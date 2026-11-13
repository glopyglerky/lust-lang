// A closure written straight into a call whose bound names the lifetime
// (`F: FnMut(&'a i32)`, `F: FnMut(&'static Message)`) takes that region
// for the argument, so its body may keep it (rustc accepts each).
struct Message {
    text: &'static str,
}
static M: Message = Message { text: "hi" };
fn apply<'a, F: FnMut(&'a i32)>(mut f: F, z: &'a i32) {
    f(z);
}
fn report<F: FnMut(&'static Message)>(mut f: F) {
    f(&M);
}
fn main() {
    let z = 3;
    let mut out: Vec<&i32> = Vec::new();
    apply(|x| out.push(x), &z);
    let mut keep: Vec<&Message> = Vec::new();
    report(|m| keep.push(m));
    println!("{} {}", out.len(), keep[0].text);
}
