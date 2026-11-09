// `&'static` written in `dyn Fn*` sugar and fn pointer types: those slots
// are `'static` (the rest stay late-bound), so a closure may keep such an
// argument, and a fn returning `'static` coerces to a pointer that does.
struct Message {
    text: &'static str,
}

static M1: Message = Message { text: "m1" };
static K: u8 = 3;

fn report(f: &mut dyn FnMut(&'static Message, Vec<String>)) {
    f(&M1, vec![String::from("a")]);
}

fn stat(_: &u8) -> &'static u8 {
    &K
}

fn push(v: &mut Vec<&'static str>, s: &'static str) {
    v.push(s);
}

fn apply(v: &mut Vec<&'static str>, f: fn(&mut Vec<&'static str>, &'static str)) {
    f(v, "x");
}

struct S {
    cb: Box<dyn Fn(&'static str) -> usize>,
}

fn main() {
    let mut seen: Vec<&'static Message> = Vec::new();
    report(&mut |m, args| {
        seen.push(m);
        println!("{} {:?}", m.text, args);
    });
    println!("{}", seen.len());
    let p: fn(&u8) -> &'static u8 = stat;
    let r;
    {
        let l = 1u8;
        r = p(&l);
    }
    println!("{}", r);
    let mut v = Vec::new();
    apply(&mut v, push);
    println!("{:?}", v);
    let s = S { cb: Box::new(|x| x.len()) };
    println!("{}", (s.cb)("hello"));
    let mut store: Vec<&'static str> = Vec::new();
    {
        let mut f: Box<dyn FnMut(&'static str)> = Box::new(|s| store.push(s));
        f("lit");
    }
    println!("{:?}", store);
}
