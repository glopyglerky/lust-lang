// MGP (ARCH 9.3): one generic Adt, one trait with an input
// parameter, one associated equality in a signature, and
// generic method calls, inherent and trait. Local types only.

// One generic Adt.
struct Bag<T> {
    items: [Option<T>; 2],
    len: usize,
}
impl<T: Copy> Bag<T> {
    fn new() -> Self {
        Bag { items: [None, None], len: 0 }
    }
    fn push(&mut self, value: T) {
        self.items[self.len] = Some(value);
        self.len += 1;
    }
    // Inherent generic method: its own type param F, fixed by
    // the closure argument and the expected Bag element type.
    fn map<U: Copy, F: Fn(T) -> U>(&self, f: F) -> Bag<U> {
        let mut out = Bag::new();
        let mut i = 0;
        while i < self.len {
            out.push(f(self.items[i].unwrap()));
            i += 1;
        }
        out
    }
    fn get(&self, index: usize) -> T {
        self.items[index].unwrap()
    }
}

// One trait with an input parameter and an associated type.
trait Convert<In> {
    type Out;
    fn convert(&self, input: In) -> Self::Out;
}
struct Doubler;
impl Convert<u8> for Doubler {
    type Out = u16;
    fn convert(&self, input: u8) -> u16 {
        input as u16 * 2
    }
}
impl Convert<u16> for Doubler {
    type Out = u32;
    fn convert(&self, input: u16) -> u32 {
        input as u32 * 2
    }
}
// A generic method on a trait: one call site must fix both the
// trait's input parameter and the method param.
trait Fold {
    fn fold_into<U: Copy>(&self, bag: &Bag<U>) -> usize;
}
impl Fold for Doubler {
    fn fold_into<U: Copy>(&self, bag: &Bag<U>) -> usize {
        bag.len
    }
}

// One associated equality used in a signature: the return type
// is u16 only through Out = u16 on the bound.
fn use_it<C: Convert<u8, Out = u16>>(c: &C, value: u8) -> u16 {
    c.convert(value) + 1
}
fn main() {
    let mut bag: Bag<u8> = Bag::new();
    bag.push(3);
    bag.push(4);
    // Inherent generic method call.
    let wide: Bag<u16> = bag.map(|x| x as u16 * 10);
    let d = Doubler;
    // Trait calls through the associated-equality signature and
    // through the second input parameter choice.
    let a = use_it(&d, 5);
    let b: u32 = d.convert(6u16);
    // Trait generic method call.
    let n = d.fold_into(&wide);
    println!("{} {} {} {} {}", wide.get(0), wide.get(1), a, b, n);
}
