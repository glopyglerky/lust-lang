// `impl Fn*` arguments keep their bounds' lifetimes like declared type
// params: a written lifetime ties the call, an elided one is higher-ranked.
fn run<'a>(v: &'a [String], f: impl Fn(&'a [String]) -> usize) -> usize {
    f(v)
}

fn each(mut f: impl FnMut(&'static str, u8)) {
    f("x", 1);
    f("y", 2);
}

fn keep<'a>(v: &'a [u8], f: impl Fn(&'a [u8]) -> &'a u8) -> &'a u8 {
    f(v)
}

fn fill<'a>(out: &mut Vec<&'a str>, src: &'a [String], mut f: impl FnMut(&'a str) -> bool) {
    for s in src {
        if f(s) {
            out.push(s);
        }
    }
}

struct S<'a> {
    v: &'a [u8],
}

impl<'a> S<'a> {
    fn pick(&self, f: impl Fn(&'a [u8]) -> &'a u8) -> &'a u8 {
        f(self.v)
    }
}

fn name(f: impl FnOnce() -> &'static str) -> usize {
    f().len()
}

fn main() {
    let v = vec![String::from("a")];
    println!("{}", run(&v, |s| s.len()));
    let mut got: Vec<&'static str> = Vec::new();
    each(|s, _| got.push(s));
    println!("{:?}", got);
    let w = vec![1u8, 2];
    let r = keep(&w, |s| &s[1]);
    println!("{}", r);
    let src = vec![String::from("ab"), String::from("c")];
    let mut out = Vec::new();
    fill(&mut out, &src, |s| s.len() > 1);
    println!("{:?}", out);
    let picked;
    {
        let s = S { v: &w };
        picked = s.pick(|v| &v[0]);
    }
    println!("{}", picked);
    let owned = String::from("abc");
    println!("{} {}", name(|| "xy"), owned);
}
