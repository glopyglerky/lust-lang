// A closure passed for `F: FnOnce(T) -> U` gives `U`'s slots only what
// its body returns there: a `'static` slot stays free of the argument,
// and an invariant slot takes the region the body needs.
pub enum Arg {
    Int(i64),
}
pub type Args = Vec<(&'static str, Arg)>;
pub struct Pop(Option<Box<PopState>>);
struct PopState {
    name: &'static str,
    args: Args,
}
impl Pop {
    pub fn args_mut(&mut self) -> Option<&mut Args> {
        self.0.as_mut().map(|state| &mut state.args)
    }
}

pub struct S<'a> {
    v: Vec<&'a str>,
    n: u8,
}
fn app<T, U>(t: T, f: impl FnOnce(T) -> U) -> U {
    f(t)
}
fn pair<'a>(s: &'a S<'a>) -> Option<(&'a u8, &'static str)> {
    Some(s).map(|st| (&st.n, "x"))
}
fn inner<'a, 'b>(s: &'b mut S<'a>) -> Option<&'b mut Vec<&'a str>> {
    Some(s).map(|st| &mut st.v)
}
fn through<'a, 'b>(s: &'b mut S<'a>) -> &'b mut Vec<&'a str> {
    app(s, |st| &mut st.v)
}

fn main() {
    let mut p = Pop(Some(Box::new(PopState { name: "ev", args: Vec::new() })));
    if let Some(a) = p.args_mut() {
        a.push(("k", Arg::Int(3)));
    }
    let st = p.0.as_ref().unwrap();
    println!("{} {}", st.name, st.args.len());
    let x = String::from("x");
    let mut s = S { v: vec![x.as_str()], n: 4 };
    println!("{:?}", pair(&s));
    inner(&mut s).unwrap().push("y");
    through(&mut s).push("z");
    println!("{:?}", s.v);
}
