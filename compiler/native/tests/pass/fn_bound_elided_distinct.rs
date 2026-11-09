// Each elided lifetime of an `Fn*` bound is its own late-bound region
// (tsrust T10: `impl FnMut(&str, &mut JsonDecoder<'_>)`), so the reborrow
// passed to `f` ends with the call.
struct Dec<'a> {
    buf: &'a [u8],
    pos: usize,
}
impl<'a> Dec<'a> {
    fn peek(&self) -> u8 {
        self.buf.get(self.pos).copied().unwrap_or(b'}')
    }
}
fn fields(dec: &mut Dec<'_>, mut field: impl FnMut(&str, &mut Dec<'_>) -> bool) -> usize {
    let mut n = 0;
    while dec.peek() != b'}' {
        let name = dec.peek().to_string();
        if !field(&name, dec) {
            dec.pos += 1;
        }
        n += 1;
    }
    n
}
fn twice<F: Fn(&mut &u8)>(f: F) -> u8 {
    let a = 1u8;
    let mut r = &a;
    f(&mut r);
    f(&mut r);
    *r
}
fn main() {
    let v = [1u8, 2, 3, b'}'];
    let mut d = Dec { buf: &v, pos: 0 };
    println!("{}", fields(&mut d, |s, d| {
        d.pos += 1;
        s.len() > 5
    }));
    static S: u8 = 9;
    println!("{}", twice(|r| *r = &S));
}
