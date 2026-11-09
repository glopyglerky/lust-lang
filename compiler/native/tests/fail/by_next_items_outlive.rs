// expect: L0306
// rustc E0597: the items a user iterator yields borrow its source.
struct W<'a> {
    s: &'a [&'a str],
    i: usize,
}

impl<'a> Iterator for W<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let w = self.s.get(self.i)?;
        self.i += 1;
        Some(*w)
    }
}

fn main() {
    let r: Vec<&str>;
    {
        let s = [String::from("a")];
        let refs: Vec<&str> = s.iter().map(|x| x.as_str()).collect();
        r = W { s: &refs[..], i: 0 }.map(|w| w).collect();
    }
    println!("{r:?}");
}
