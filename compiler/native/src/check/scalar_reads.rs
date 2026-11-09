// Const fn reads of the scalar evaluator (collect.rs `ScalarEval`). A body
// check asks it for array lengths (`[0; k()]`, `[u8; k()]`) and const
// generic arguments before the full evaluator (`CheckCx`), and it runs
// const fn bodies on its own, by written name, with no read sink. The
// result is baked into the body, so the body depends on those const fn
// bodies: each call the evaluator makes while a scope is open notes the
// callee's name in a per-thread log, and `check_fn` stores one
// `ReadKey::Checked` read per local const fn of a noted name, as
// `CheckCx::const_fn_call` does for the full evaluator. A `Checked` read
// never replays clean, so the body rechecks whenever it is a read-set
// walk candidate (a const fn body edit makes every passing body one).
use crate::check::collect::{LocalDefId, Program};
use crate::driver::graph::{Fingerprint, Reads};
use crate::intern::Sym;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Open scopes on this thread; zero means nothing is logged.
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    /// The callee names noted since the outermost scope opened.
    static LOG: RefCell<Vec<Sym>> = const { RefCell::new(Vec::new()) };
}

/// Notes a call of a fn named `name` by the scalar evaluator.
pub(crate) fn note(name: Sym) {
    if DEPTH.with(Cell::get) == 0 {
        return;
    }
    LOG.with(|l| l.borrow_mut().push(name));
}

/// Runs `f` with a scope open (when `on`) and returns the names noted in
/// it, sorted and deduplicated.
pub(crate) fn scoped<R>(on: bool, f: impl FnOnce() -> R) -> (R, Vec<Sym>) {
    if !on {
        return (f(), Vec::new());
    }
    let start = LOG.with(|l| l.borrow().len());
    DEPTH.with(|d| d.set(d.get() + 1));
    // The guard closes the scope on unwind too (a checker panic caught
    // by the daemon must not leave this thread logging).
    struct Close;
    impl Drop for Close {
        fn drop(&mut self) {
            let depth = DEPTH.with(|d| {
                d.set(d.get() - 1);
                d.get()
            });
            if depth == 0 {
                LOG.with(|l| l.borrow_mut().clear());
            }
        }
    }
    let close = Close;
    let out = f();
    let mut noted = LOG.with(|l| l.borrow()[start..].to_vec());
    drop(close);
    noted.sort_unstable();
    noted.dedup();
    (out, noted)
}

/// Stores a `Checked` read for every local const fn with a body whose
/// name is in `names` (the evaluator resolves by written name, so every
/// fn of the name counts).
pub(crate) fn record(p: &Program, reads: &mut Reads, names: &[Sym]) {
    if names.is_empty() {
        return;
    }
    for i in 0..p.fns.len() {
        let f = LocalDefId(i as u32);
        let info = p.fn_info(f);
        if info.is_const && names.binary_search(&info.name).is_ok() && p.fn_def(f).body.is_some() {
            reads.record_checked(p.def_id(f), Fingerprint(0));
        }
    }
}
