// expect: L0306
// rustc E0597: `dyn FnMut(&'static Message)` takes only `'static` borrows.
struct Message {
    text: &'static str,
}

fn report(f: &mut dyn FnMut(&'static Message)) {
    let m = Message { text: "local" };
    f(&m);
}

fn main() {}
