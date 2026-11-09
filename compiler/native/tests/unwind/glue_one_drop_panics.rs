// Drop glue with one drop and nothing after it (an Option, a one-field struct, an enum with
// one droppable arm) has no cleanup frame: a panic in that drop leaves the glue with nothing
// to drop. The panic still goes on, and glue around it with more fields still drops them.
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

trait T {
    fn id(&self) -> i32;
}
impl T for P {
    fn id(&self) -> i32 {
        self.0
    }
}

#[allow(dead_code)]
struct One {
    p: P,
}
#[allow(dead_code)]
struct Wrap(Option<P>);
#[allow(dead_code)]
enum E {
    A(P),
    B(i32),
}
#[allow(dead_code)]
struct Outer {
    a: Option<P>,
    b: P,
    c: One,
}

fn run(name: &str, f: impl FnOnce() + panic::UnwindSafe) {
    let r = panic::catch_unwind(f);
    let msg = match &r {
        Ok(()) => "ok".to_string(),
        Err(e) => e.downcast_ref::<String>().cloned().unwrap_or_else(|| "?".into()),
    };
    println!("{name}: {msg}");
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    run("option", || drop(Some(P(2))));
    run("option none", || drop(None::<P>));
    run("one field", || drop(One { p: P(12) }));
    run("wrap", || drop(Wrap(Some(P(22)))));
    run("enum arm", || drop(E::A(P(32))));
    run("rc dyn", || {
        let r: Option<Rc<dyn T>> = Some(Rc::new(P(42)));
        println!("id {}", r.as_ref().map_or(0, |r| r.id()));
        drop(r);
    });
    run("rc dyn shared", || {
        let r: Rc<dyn T> = Rc::new(P(52));
        let s = Some(r.clone());
        drop(s);
        println!("still {}", r.id());
    });
    run("outer a", || drop(Outer { a: Some(P(62)), b: P(63), c: One { p: P(64) } }));
    run("outer c", || drop(Outer { a: Some(P(71)), b: P(73), c: One { p: P(72) } }));
    run("vec of options", || drop(vec![Some(P(81)), Some(P(82)), None, Some(P(83))]));
    run("no panic", || drop(Outer { a: None, b: P(91), c: One { p: P(93) } }));
}
