// expect: L0306
// rustc E0597: a frame of a decoder over `text` cannot hold a name that
// borrows a shorter-lived string.
struct Frame<'a> {
    names: Vec<&'a str>,
}

struct Decoder<'a> {
    buf: &'a str,
    stack: Vec<Frame<'a>>,
}

impl<'a> Decoder<'a> {
    fn push_name(&mut self, n: &'a str) {
        if let Some(top) = self.stack.last_mut() {
            top.names.push(n);
        }
    }
}

fn main() {
    let text = String::from("abc");
    let mut d = Decoder { buf: &text, stack: vec![Frame { names: Vec::new() }] };
    {
        let local = String::from("short");
        d.push_name(&local);
    }
    println!("{} {}", d.buf, d.stack.len());
}
