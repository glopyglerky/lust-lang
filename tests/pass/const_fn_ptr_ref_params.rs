// Fn items with elided reference params as fn pointer values in
// consts and statics (ts-rust collate options, norm form table).
#[derive(Clone, Copy)]
enum Form {
    Nfc,
    Nfd,
}
const NFC: Form = Form::Nfc;
const NFD: Form = Form::Nfd;
struct Properties {
    pos: u8,
    size: u8,
}
type LookupFunc = fn(b: &[u8], i: usize) -> Properties;
struct FormInfo {
    form: Form,
    composing: bool,
    info: LookupFunc,
}
fn lookup(b: &[u8], i: usize) -> Properties {
    Properties { pos: b[i], size: 1 }
}
static FORM_TABLE: [FormInfo; 2] = [
    FormInfo { form: NFC, composing: true, info: lookup },
    FormInfo { form: NFD, composing: false, info: lookup },
];
struct Opt {
    priority: i32,
    f: fn(&mut i32),
}
fn inc(x: &mut i32) {
    *x += 1;
}
fn dbl(x: &mut i32) {
    *x *= 2;
}
const INC: Opt = Opt { priority: 3, f: inc };
static OPTS: [Opt; 2] = [Opt { priority: 1, f: inc }, Opt { priority: 2, f: dbl }];
fn alen(s: &str) -> usize {
    s.len()
}
static NAMED: [(&str, fn(&str) -> usize); 1] = [("alen", alen)];
fn main() {
    let p = (FORM_TABLE[1].info)(&[7, 8], 1);
    println!("{} {} {} {}", p.pos, p.size, FORM_TABLE[0].composing, FORM_TABLE[1].form as u8);
    let mut v = 5;
    (INC.f)(&mut v);
    for o in &OPTS {
        (o.f)(&mut v);
        println!("{} {}", o.priority, v);
    }
    println!("{} {}", INC.priority, v);
    for (n, f) in &NAMED {
        println!("{} {}", n, f("hello"));
    }
}
