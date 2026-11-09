// A written `!` is stable as a whole return type: of a fn, a method, a
// trait method, a closure, and a fn pointer (also inside other types,
// as `Vec<fn() -> !>`), and in code that `cfg` removes.
fn stop(code: i32) -> ! {
    println!("stop {code}");
    std::process::exit(code)
}

struct S;
impl S {
    fn halt(&self) -> ! {
        stop(3)
    }
}

trait Halt {
    fn halt_now(&self) -> !;
}
impl Halt for S {
    fn halt_now(&self) -> ! {
        self.halt()
    }
}

#[cfg(any())]
fn gone(_x: !) -> Option<!> {
    None
}

fn main() {
    let fs: Vec<fn(i32) -> !> = vec![stop];
    let c = |n: i32| -> ! { fs[0](n) };
    if std::env::args().count() > 5 {
        c(1);
    }
    let g: Option<fn() -> !> = None;
    println!("{}", g.is_none());
    S.halt_now();
}
