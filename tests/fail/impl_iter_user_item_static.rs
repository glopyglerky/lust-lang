// expect: L0733
// rustc: lifetime may not live long enough. A user iterator's items are
// its `'a` borrow of `self`, not `'static`.
struct It<'a> {
    v: &'a [String],
    i: usize,
}
impl<'a> Iterator for It<'a> {
    type Item = &'a String;
    fn next(&mut self) -> Option<&'a String> {
        let r = self.v.get(self.i);
        self.i += 1;
        r
    }
}
struct S {
    v: Vec<String>,
}
impl S {
    fn names(&self) -> impl Iterator<Item = &'static String> + '_ {
        It { v: &self.v, i: 0 }
    }
}
fn main() {
    let s = S { v: vec!["a".to_string()] };
    for n in s.names() {
        println!("{}", n);
    }
}
