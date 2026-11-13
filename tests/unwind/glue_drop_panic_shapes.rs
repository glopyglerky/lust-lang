// A drop that panics inside drop glue of each shape: the rest of the value still drops
// (as Rust's landing pads do), then the panic goes on to catch_unwind. Enum variant
// fields, Vec and array elements, a Box<dyn Trait>, an Rc, an Option, a tuple, a struct
// with its own Drop, and a Vec of structs.
use std::panic;
use std::rc::Rc;

struct P(i32);
impl Drop for P {
    fn drop(&mut self) {
        println!("drop {}", self.0);
        if self.0 % 10 == 2 {
            panic!("drop panic {}", self.0);
        }
    }
}

enum E {
    Two(P, P, P),
    #[allow(dead_code)]
    One(P),
}

trait T {
    fn id(&self) -> i32;
}
impl T for P {
    fn id(&self) -> i32 {
        self.0
    }
}

struct Outer {
    a: P,
    v: Vec<P>,
    b: P,
}
impl Drop for Outer {
    fn drop(&mut self) {
        println!("drop outer {}", self.a.0);
    }
}

fn run(name: &str, f: impl FnOnce() + panic::UnwindSafe) {
    let r = panic::catch_unwind(f);
    println!("{name}: {}", if r.is_err() { "caught" } else { "ok" });
}

fn main() {
    run("enum", || {
        let _e = E::Two(P(11), P(12), P(13));
    });
    run("vec", || {
        let _v = vec![P(21), P(22), P(23), P(24)];
    });
    run("array", || {
        let _a = [P(31), P(32), P(33)];
    });
    run("box dyn", || {
        let b: Box<dyn T> = Box::new(P(42));
        println!("id {}", b.id());
    });
    run("rc", || {
        let r = Rc::new(P(52));
        let r2 = r.clone();
        drop(r);
        println!("strong {}", Rc::strong_count(&r2));
    });
    run("option", || {
        let _o = Some((P(61), P(62), P(63)));
    });
    run("tuple", || {
        let _t = (P(71), P(72), P(73));
    });
    run("outer", || {
        let _o = Outer { a: P(81), v: vec![P(91), P(92), P(93)], b: P(83) };
    });
    run("vec of outer", || {
        let _v = vec![Outer { a: P(101), v: vec![], b: P(103) }, Outer { a: P(105), v: vec![P(112)], b: P(106) }];
    });
    println!("done");
}
