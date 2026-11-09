// `core::mem::ManuallyDrop` (TRIAGE R2): no drop glue (values, Option, Vec,
// struct fields), into_inner, Deref/DerefMut method calls and fields, the
// derives and cmp impls, Debug text, consts and statics, layout, needs_drop.
use std::mem::ManuallyDrop;
use std::rc::Rc;

struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct P(i32);

enum Repr {
    A(&'static [u32]),
    B(Rc<[u32]>),
    C { len: u8, items: [u32; 3] },
}

struct List {
    repr: ManuallyDrop<Repr>,
}

impl Drop for List {
    fn drop(&mut self) {
        if matches!(*self.repr, Repr::B(_)) {
            drop_owned(&mut self.repr);
        }
    }
}

fn drop_owned(r: &mut Repr) {
    drop(std::mem::replace(r, Repr::A(&[])));
}

fn len(l: &List) -> usize {
    match &*l.repr {
        Repr::A(s) => s.len(),
        Repr::B(s) => s.len(),
        Repr::C { len, .. } => *len as usize,
    }
}

const C: ManuallyDrop<i32> = ManuallyDrop::new(7);
static S: ManuallyDrop<&str> = ManuallyDrop::new("st");

fn main() {
    let a = ManuallyDrop::new(Noisy("a"));
    println!("made {}", a.0);
    {
        let _b = ManuallyDrop::new(Noisy("b"));
    }
    println!("after b");
    let c = ManuallyDrop::new(Noisy("c"));
    let c2 = ManuallyDrop::into_inner(c);
    println!("into_inner {}", c2.0);
    drop(c2);
    let mut v = ManuallyDrop::new(vec![1, 2, 3]);
    v.push(4);
    println!("{:?} {}", *v, v.len());
    // A heap value must leave its ManuallyDrop (no leak under SANITIZE).
    drop(ManuallyDrop::into_inner(v));
    let o: Option<ManuallyDrop<Noisy>> = Some(ManuallyDrop::new(Noisy("o")));
    drop(o);
    let vs = vec![ManuallyDrop::new(Noisy("v1")), ManuallyDrop::new(Noisy("v2"))];
    drop(vs);
    let rc: Rc<[u32]> = Rc::from(&[1u32, 2, 3, 4][..]);
    let l = List { repr: ManuallyDrop::new(Repr::B(rc.clone())) };
    println!("rc {} len {}", Rc::strong_count(&rc), len(&l));
    drop(l);
    println!("rc {}", Rc::strong_count(&rc));
    let l2 = List { repr: ManuallyDrop::new(Repr::C { len: 2, items: [1, 2, 0] }) };
    println!("len {}", len(&l2));
    let p = ManuallyDrop::new(P(3));
    let q = p;
    println!("{:?} {:?} {} {} {:?}", p, q, p == q, p < ManuallyDrop::new(P(4)), p.cmp(&q));
    println!("{:?} {:?} {}", C, *S, *C + 1);
    let d: ManuallyDrop<P> = Default::default();
    println!("{:?} {:?}", d, d.clone());
    println!("{} {}", std::mem::size_of::<ManuallyDrop<u64>>(), std::mem::size_of::<Option<ManuallyDrop<Box<u8>>>>());
    println!("{} {}", std::mem::needs_drop::<ManuallyDrop<String>>(), std::mem::needs_drop::<(ManuallyDrop<String>, u8)>());
    let mut m = ManuallyDrop::new(String::from("hi"));
    m.push_str(" there");
    let s: &str = m.as_str();
    println!("{} {}", s, m.to_uppercase());
    let inner = ManuallyDrop::into_inner(m);
    println!("{}", inner);
    let _ = len(&l2);
    println!("end");
}
