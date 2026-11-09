// `cond.then(|| ..)` holds what the closure returns, not every region it
// captures, and a closure passed only for `FnOnce` may return a reborrow
// of its `&mut` capture (rustc takes the kind from the bound).
#[derive(Clone, Copy)]
pub struct R {
    flow: usize,
    file: u32,
}
impl R {
    fn is_for(self, f: u32) -> bool {
        self.file == f
    }
}
pub struct A {
    nodes: Vec<u8>,
    file: u32,
}
impl A {
    pub fn get(&self, reference: R) -> Option<&u8> {
        reference.is_for(self.file).then(|| self.nodes.get(reference.flow)).flatten()
    }
    pub fn get_mut(&mut self, reference: R) -> Option<&mut u8> {
        reference.is_for(self.file).then(|| self.nodes.get_mut(reference.flow)).flatten()
    }
}
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn once<F: FnOnce() -> u8>(f: F) -> u8 {
    let r = f();
    println!("after");
    r
}
fn main() {
    let mut a = A { nodes: vec![1, 2], file: 0 };
    let r = R { flow: 1, file: 0 };
    println!("{:?}", a.get(r));
    *a.get_mut(r).unwrap() = 5;
    println!("{:?}", a.nodes);
    let d = D(1);
    let mut v = vec![1u8];
    println!("{}", once(|| {
        v.push(2);
        d.0
    }));
    let e = D(2);
    println!("{}", once(move || e.0 + 1));
    let mut w = vec![5u8];
    let last = true.then(|| w.last_mut().unwrap());
    *last.unwrap() = 9;
    println!("{:?} {:?}", v, w);
}
