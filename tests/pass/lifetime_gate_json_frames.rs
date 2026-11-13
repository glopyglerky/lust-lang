// The json kernel's shape (tests/kernels/json): a decoder over a borrowed
// buffer with a stack of frames whose names borrow the same buffer.
struct Frame<'a> {
    is_object: bool,
    names: Vec<&'a str>,
}

struct Decoder<'a> {
    buf: &'a str,
    pos: usize,
    stack: Vec<Frame<'a>>,
}

impl<'a> Decoder<'a> {
    fn new(buf: &'a str) -> Decoder<'a> {
        Decoder { buf, pos: 0, stack: Vec::new() }
    }

    fn open(&mut self, is_object: bool) {
        self.stack.push(Frame { is_object, names: Vec::new() });
    }

    /// Reads a name of `len` bytes; it borrows the buffer, not the decoder.
    fn name(&mut self, len: usize) -> &'a str {
        let s = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        if let Some(top) = self.stack.last_mut() {
            top.names.push(s);
        }
        s
    }

    fn close(&mut self) -> Option<Frame<'a>> {
        self.stack.pop()
    }
}

fn main() {
    let text = String::from("keyvalue");
    let mut d = Decoder::new(&text);
    d.open(true);
    let k = d.name(3);
    let v = d.name(5);
    let f = d.close().unwrap();
    println!("{} {} {} {:?}", k, v, f.is_object, f.names);
}
