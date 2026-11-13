// macro: trace_write, args
// source: crates/ts_goport/src/frontend/module/resolver_p1.rs:24
// source: crates/ts_goport/src/core.rs:3836
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::cell::RefCell;

#[macro_export]
macro_rules! args {
    ($($arg:expr),* $(,)?) => { vec![$(::std::string::ToString::to_string(&$arg)),*] };
}

pub struct Tracer { pub log: Vec<String> }
impl Tracer {
    pub fn write(&mut self, msg: &str, args: Vec<String>) { self.log.push(format!("{msg}:{args:?}")); }
}
pub struct State { pub tracer: Option<RefCell<Tracer>> }

macro_rules! trace_write {
    ($state:expr, $msg:expr $(, $arg:expr)* $(,)?) => {
        if let Some(tracer) = &$state.tracer {
            tracer.borrow_mut().write($msg, args![$($arg),*]);
        }
    };
}

fn main() {
    let state = State { tracer: Some(RefCell::new(Tracer { log: Vec::new() })) };
    trace_write!(state, "resolve", "a.ts", 2);
    trace_write!(state, "done");
    let quiet = State { tracer: None };
    trace_write!(quiet, "never", 1);
    if let Some(t) = &state.tracer { println!("log={:?}", t.borrow().log); }
}
