// ARCH 2.10 steps 1 and 2: autoderef walks Deref steps, and
// autoref probes &S and &mut S forms on a value receiver.
struct Inner {
    n: u8,
}
impl Inner {
    fn get(&self) -> u8 {
        self.n
    }
    fn bump(&mut self) {
        self.n += 1;
    }
}
struct Wrap(Box<Inner>);
impl std::ops::Deref for Wrap {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}
impl std::ops::DerefMut for Wrap {
    fn deref_mut(&mut self) -> &mut Inner {
        &mut self.0
    }
}
fn main() {
    let mut w = Wrap(Box::new(Inner { n: 1 }));
    // get needs one Deref step plus autoref on a place.
    let a = w.get();
    // bump needs DerefMut plus a &mut autoref.
    w.bump();
    // Two boxed steps: Box<Box<Inner>> reaches Inner methods.
    let deep = Box::new(Box::new(Inner { n: 9 }));
    // String reaches str methods through Deref.
    let text = String::from("abcd");
    println!("{} {} {} {}", a, w.get(), deep.get(), text.len());
}
