// A `&'static str` returned through `&dyn Name` borrows nothing: the
// object can die before the result is used (the trait's signature).
trait Name {
    fn name(&self) -> &'static str;
}

struct A {
    s: &'static str,
}

impl Name for A {
    fn name(&self) -> &'static str {
        self.s
    }
}

fn get(d: &dyn Name) -> usize {
    d.name().len()
}

fn main() {
    let r;
    {
        let a = A { s: "a" };
        let d: &dyn Name = &a;
        r = d.name();
        println!("{}", get(d));
    }
    println!("{}", r);
}
