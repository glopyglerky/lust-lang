// Static drop flags: rustc's drop elaboration, for whole locals. Codegen gives a local whose type
// needs drop a flag `d{l}` that says whether it still owns a value: a move of the whole local
// clears it, an assignment to the whole local sets it, and each `Drop` (at scope exit or in a
// cleanup path) and each assignment's drop of the old value tests it. Most of those tests see the
// same value on every path that reaches them; this pass finds them, so codegen gives such a flag
// no storage, never stores it, and drops there unconditionally or not at all. Only flags whose
// value at some test depends on the path stay runtime flags. cgen and the LLVM backend both read
// the result; the LLVM backend's default leaves it off (`cgen::static_drop_flags`).
//
// A forward dataflow over the body's blocks and their call and statement unwind edges, with
// cgen's flag rules (`FnGen::operand`, `drop_old_value`, `emit_assign`, `StmtKind::Drop`):
//   - a parameter's flag starts set, every other flag clear;
//   - an assignment clears the flags of the whole locals its value moves, at its end; a call
//     clears its moved arguments before it runs, so its unwind edge sees them clear; a branch,
//     a print and a panic message read their operands and clear nothing;
//   - an assignment to a whole local tests its flag after the statement's moves (the old value
//     drops if set) and then sets it; a call does that on its normal edge only;
//   - a `Drop` tests the flag, then clears it before the drop runs.
// A statement's or call's unwind edge carries every state a panic inside it can leave: its start
// (when making the value or an indexed destination's place can panic), after its moves, and with
// its destination's flag clear for the old value's drop. That covers both unwinding modes (with
// unwind tables a panic leaves from inside the statement; with flags, from the check after the
// call that panicked).
//
// For the C backends (`fine`) a flag that stays a runtime flag is elaborated per test too: a test
// that sees one value on every path folds to a constant, as rustc folds each drop site, and only
// the tests that see either value read the flag. A backward liveness over the same edges then
// finds the stores no such test can read before the next store (the entry value, a move's clear,
// an assignment's set, the clear before a drop that always runs), and codegen leaves them out:
// clang cannot, because the frame struct holding the flag escapes to the cleanup.
// Move flags (`m{l}_{k}`, partial moves) are not changed here.
use std::collections::{HashMap, HashSet};

use super::{Body, Local, Operand, Place, Proj, Rvalue, StmtKind, Terminator, rvalue_moves};

/// No path reaches here yet; the flag is clear; set; or either, depending on the path.
const NONE: u8 = 0;
const CLEAR: u8 = 1;
const SET: u8 = 2;
const DYN: u8 = 3;

fn meet(a: u8, b: u8) -> u8 {
    if a == NONE || a == b {
        b
    } else if b == NONE {
        a
    } else {
        DYN
    }
}

/// What codegen needs to know about one body's whole-local drop flags.
pub struct DropFlags {
    /// Per local: its flag stays a runtime flag (a test sees a value that depends on the path).
    /// A local with no flag counts as kept: codegen treats it as before this pass.
    dynamic: Vec<bool>,
    /// (block, statement index) -> the local tested there and the value its test sees (CLEAR,
    /// SET or DYN). An index equal to the block's statement count names its `Call` terminator.
    tests: HashMap<(u32, u32), (Local, u8)>,
    /// Runtime flags too fold their static tests and leave out their dead stores (C backends).
    fine: bool,
    /// Per local: its flag's entry value can be read by a test (`fine` only).
    entry_live: Vec<bool>,
    /// Stores of a runtime flag no test reads before the next store (`fine` only).
    dead: HashSet<(u32, u32, Local, Store)>,
}

/// A store to a runtime flag, by where codegen makes it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Store {
    /// A move of the whole local clears it (`FnGen::operand`).
    Clear,
    /// An assignment to the whole local sets it after the old value (`drop_old_value`).
    Set,
    /// A drop that runs on every path clears it first: a `Drop`, an assignment's old value.
    DropClear,
}

impl DropFlags {
    /// Every flag a runtime flag: codegen as before this pass (`LUST_DROPFLAGS=0`).
    pub fn all_runtime(n_locals: usize) -> DropFlags {
        DropFlags { dynamic: vec![true; n_locals], tests: HashMap::new(), fine: false, entry_live: Vec::new(), dead: HashSet::new() }
    }

    /// The local keeps a runtime flag (codegen declares, stores and tests `d{l}`).
    pub fn keeps(&self, l: Local) -> bool {
        self.dynamic.get(l as usize).copied().unwrap_or(true)
    }

    /// The value the test of `l`'s flag at (block, statement index) sees when it is the same on
    /// every path: true drops, false does not. None: a runtime test of a runtime flag.
    pub fn test(&self, at: (u32, u32), l: Local) -> Option<bool> {
        match self.tests.get(&at) {
            Some(&(t, v)) if t == l => (v != DYN && (self.fine || !self.keeps(l))).then_some(v == SET),
            _ if self.keeps(l) => None,
            _ => panic!("drop flag of l{l} tested at {at:?} with no static value"),
        }
    }

    /// Codegen makes this store to `l`'s runtime flag at (block, statement index).
    pub fn store(&self, at: (u32, u32), l: Local, kind: Store) -> bool {
        self.keeps(l) && !self.dead.contains(&(at.0, at.1, l, kind))
    }

    /// Codegen gives `l`'s runtime flag its entry value.
    pub fn entry(&self, l: Local) -> bool {
        self.keeps(l) && self.entry_live.get(l as usize).copied().unwrap_or(true)
    }
}

/// The drop flags of `body`. `flagged[l]`: codegen gives local `l` a flag (its type needs drop).
/// `fine`: also fold the static tests of runtime flags and find their dead stores.
pub fn analyze(body: &Body, flagged: &[bool], fine: bool) -> DropFlags {
    let n = body.locals.len();
    let mut idx = vec![usize::MAX; n];
    let mut m = 0;
    for (l, slot) in idx.iter_mut().enumerate() {
        if flagged.get(l).copied().unwrap_or(false) {
            *slot = m;
            m += 1;
        }
    }
    if m == 0 || body.blocks.is_empty() {
        return DropFlags::all_runtime(n);
    }
    let whole = |p: &Place| if p.proj.is_empty() { idx.get(p.local as usize).copied().filter(|&k| k != usize::MAX) } else { None };
    let clear_moves = |s: &mut [u8], moved: &[&Place]| {
        for p in moved {
            if let Some(k) = whole(p) {
                s[k] = CLEAR;
            }
        }
    };
    let operand_clear = |s: &mut [u8], o: &Operand| {
        if let Operand::Move(p) = o
            && let Some(k) = whole(p)
        {
            s[k] = CLEAR;
        }
    };
    let meet_all = |a: &[u8], b: &[u8]| a.iter().zip(b).map(|(&x, &y)| meet(x, y)).collect::<Vec<u8>>();
    let unw: HashMap<(u32, u32), usize> = body.stmt_unwind.iter().map(|&(b, i, u)| ((b.0, i), u.0 as usize)).collect();
    let nb = body.blocks.len();
    let mut entry: Vec<Option<Vec<u8>>> = vec![None; nb];
    let mut init = vec![CLEAR; m];
    for l in 1..=body.arg_count.min(n.saturating_sub(1)) {
        if idx[l] != usize::MAX {
            init[idx[l]] = SET;
        }
    }
    entry[0] = Some(init);
    let mut work = vec![0usize];
    let mut queued = vec![false; nb];
    queued[0] = true;
    // (block, statement index) -> (tested local, value seen), from each block's last visit.
    let mut tests: HashMap<(u32, u32), (Local, u8)> = HashMap::new();
    while let Some(b) = work.pop() {
        queued[b] = false;
        let mut s = entry[b].clone().expect("queued blocks have a state");
        let block = &body.blocks[b];
        let mut outs: Vec<(usize, Vec<u8>)> = Vec::new();
        for (si, st) in block.stmts.iter().enumerate() {
            let edge = unw.get(&(b as u32, si as u32)).copied();
            match &st.kind {
                StmtKind::Assign(dest, rv) => {
                    let before = edge.map(|_| s.clone());
                    let mut moved = Vec::new();
                    rvalue_moves(rv, &mut moved);
                    clear_moves(&mut s, &moved);
                    let d = whole(dest);
                    if let Some(k) = d {
                        tests.insert((b as u32, si as u32), (dest.local, s[k]));
                    }
                    if let (Some(u), Some(before)) = (edge, before) {
                        // Where a panic inside can leave: while the value or an indexed
                        // destination's place is made (only when that can panic), else after the
                        // moves; the old value's drop panics with the destination's flag already
                        // clear.
                        let mut e = if may_panic_before_moves(dest, rv) { meet_all(&before, &s) } else { s.clone() };
                        if let Some(k) = d {
                            e[k] = meet(e[k], CLEAR);
                        }
                        outs.push((u, e));
                    }
                    if let Some(k) = d {
                        s[k] = SET;
                    }
                }
                StmtKind::Drop(l) => {
                    if let Some(k) = whole(&Place::local(*l)) {
                        tests.insert((b as u32, si as u32), (*l, s[k]));
                        s[k] = CLEAR;
                    }
                    if let Some(u) = edge {
                        outs.push((u, s.clone()));
                    }
                }
                // A print reads its arguments; like a branch and a panic, it clears no flag.
                StmtKind::Print { .. } | StmtKind::StorageDead(_) | StmtKind::FakeRead(_) => {
                    if let Some(u) = edge {
                        outs.push((u, s.clone()));
                    }
                }
            }
        }
        let nst = block.stmts.len() as u32;
        // A terminator's cleanup entry (a panic message's; a call has its own unwind
        // successor, which codegen uses instead): no flag changes before it can panic.
        if let Some(&u) = unw.get(&(b as u32, nst)) {
            outs.push((u, s.clone()));
        }
        match &block.term {
            Terminator::Goto(t) => outs.push((t.0 as usize, s)),
            // A branch on a moved operand clears no flag (cgen emits no moves for it).
            Terminator::If(_, a, c) => {
                outs.push((a.0 as usize, s.clone()));
                outs.push((c.0 as usize, s));
            }
            Terminator::Switch(_, cases, other) => {
                for (_, t) in cases {
                    outs.push((t.0 as usize, s.clone()));
                }
                outs.push((other.0 as usize, s));
            }
            Terminator::Call { args, dest, next, unwind, .. } => {
                for a in args {
                    operand_clear(&mut s, a);
                }
                if let Some(u) = unwind {
                    // The old value's drop after the call panics to the same cleanup, with the
                    // destination's flag already clear.
                    let mut e = s.clone();
                    if let Some(k) = whole(dest) {
                        e[k] = meet(e[k], CLEAR);
                    }
                    outs.push((u.0 as usize, e));
                }
                if let Some(k) = whole(dest) {
                    tests.insert((b as u32, nst), (dest.local, s[k]));
                    s[k] = SET;
                }
                outs.push((next.0 as usize, s));
            }
            Terminator::Panic(_) | Terminator::Return | Terminator::Unreachable | Terminator::MatchExhausted => {}
        }
        for (t, st) in outs {
            let grew = match &mut entry[t] {
                Some(e) => {
                    let mut changed = false;
                    for (x, &y) in e.iter_mut().zip(&st) {
                        let v = meet(*x, y);
                        changed |= v != *x;
                        *x = v;
                    }
                    changed
                }
                None => {
                    entry[t] = Some(st);
                    true
                }
            };
            if grew && !queued[t] {
                queued[t] = true;
                work.push(t);
            }
        }
    }
    let mut dynamic: Vec<bool> = idx.iter().map(|&k| k == usize::MAX).collect();
    for &(l, v) in tests.values() {
        if v == DYN || v == NONE {
            dynamic[l as usize] = true;
        }
    }
    let (entry_live, dead) = if fine { liveness(body, &dynamic, &idx, &tests, &unw) } else { (Vec::new(), HashSet::new()) };
    DropFlags { dynamic, tests, fine, entry_live, dead }
}

/// A set of runtime flags, by their index among a body's runtime flags.
#[derive(Clone, PartialEq)]
struct Bits(Vec<u64>);

impl Bits {
    fn new(n: usize) -> Bits {
        Bits(vec![0; n.div_ceil(64)])
    }
    fn has(&self, k: usize) -> bool {
        self.0[k / 64] >> (k % 64) & 1 == 1
    }
    fn add(&mut self, k: usize) {
        self.0[k / 64] |= 1 << (k % 64);
    }
    fn del(&mut self, k: usize) {
        self.0[k / 64] &= !(1 << (k % 64));
    }
    fn or(&mut self, o: &Bits) {
        self.0.iter_mut().zip(&o.0).for_each(|(a, b)| *a |= b);
    }
    fn or_but(&mut self, o: &Bits, k: usize) {
        let had = self.has(k);
        self.or(o);
        if !had {
            self.del(k);
        }
    }
}

/// Backward liveness of the runtime flags (`dynamic`): a flag is live where a runtime test (one
/// whose value in `tests` is DYN: a `Drop`, an old-value drop, also in a cleanup reached by an
/// unwind edge) may read it before the next store. Returns per local whether its entry value is
/// live, and the dead stores. A folded test reads nothing; a drop that runs on every path clears
/// the flag first (`Store::DropClear`), which a cleanup reached from inside that drop reads.
fn liveness(
    body: &Body,
    dynamic: &[bool],
    idx: &[usize],
    tests: &HashMap<(u32, u32), (Local, u8)>,
    unw: &HashMap<(u32, u32), usize>,
) -> (Vec<bool>, HashSet<(u32, u32, Local, Store)>) {
    let n = body.locals.len();
    // Runtime flags only, by a dense index.
    let mut kidx = vec![usize::MAX; n];
    let mut local_of = Vec::new();
    for l in 0..n {
        if dynamic[l] && idx[l] != usize::MAX {
            kidx[l] = local_of.len();
            local_of.push(l as Local);
        }
    }
    let m = local_of.len();
    if m == 0 {
        return (vec![false; n], HashSet::new());
    }
    let rt = |p: &Place| if p.proj.is_empty() { kidx.get(p.local as usize).copied().filter(|&k| k != usize::MAX) } else { None };
    let nb = body.blocks.len();
    let mut preds = vec![Vec::new(); nb];
    for (b, block) in body.blocks.iter().enumerate() {
        for t in super::succ(&block.term) {
            preds[t.0 as usize].push(b);
        }
    }
    for &(b, _, u) in &body.stmt_unwind {
        preds[u.0 as usize].push(b.0 as usize);
    }
    let mut live_in = vec![Bits::new(m); nb];
    let mut dead = HashSet::new();
    // The value the test at a position sees, for its local.
    let tv = |b: u32, i: u32, l: Local| tests.get(&(b, i)).filter(|(t, _)| *t == l).map(|&(_, v)| v);
    let block_live = |b: usize, live_in: &[Bits], dead: &mut Option<&mut HashSet<(u32, u32, Local, Store)>>| -> Bits {
        let block = &body.blocks[b];
        let bu = b as u32;
        let nst = block.stmts.len() as u32;
        let mut mark = |at: u32, l: Local, kind: Store, live: bool| {
            if !live && let Some(d) = dead.as_deref_mut() {
                d.insert((bu, at, l, kind));
            }
        };
        let edge = |i: u32| unw.get(&(bu, i)).map(|&u| &live_in[u]);
        let mut live = Bits::new(m);
        // An old value's drop at (block, index) of a flagged `dest`, after the statement's moves:
        // `live` is what is live after the set that follows it.
        let old_value = |live: &mut Bits, at: u32, dest: &Place, cleanup: Option<&Bits>, mark: &mut dyn FnMut(u32, Local, Store, bool)| {
            let k = rt(dest);
            if let Some(k) = k {
                mark(at, dest.local, Store::Set, live.has(k));
                live.del(k);
            }
            match tv(bu, at, dest.local) {
                // A runtime test: the old value's drop unwinds with the flag already clear.
                Some(DYN) | Some(NONE) => match (cleanup, k) {
                    (Some(c), Some(k)) => {
                        live.or_but(c, k);
                        live.add(k);
                    }
                    (Some(c), None) => live.or(c),
                    (None, Some(k)) => live.add(k),
                    (None, None) => {}
                },
                Some(SET) => {
                    if let Some(c) = cleanup {
                        if let Some(k) = k {
                            mark(at, dest.local, Store::DropClear, c.has(k));
                            live.or_but(c, k);
                        } else {
                            live.or(c);
                        }
                    } else if k.is_some() {
                        mark(at, dest.local, Store::DropClear, false);
                    }
                }
                Some(_) => {}
                None => {
                    if let Some(c) = cleanup {
                        live.or(c);
                    }
                }
            }
        };
        match &block.term {
            Terminator::Goto(t) => live.or(&live_in[t.0 as usize]),
            Terminator::If(_, a, c) => {
                live.or(&live_in[a.0 as usize]);
                live.or(&live_in[c.0 as usize]);
            }
            Terminator::Switch(_, cases, other) => {
                for (_, t) in cases {
                    live.or(&live_in[t.0 as usize]);
                }
                live.or(&live_in[other.0 as usize]);
            }
            Terminator::Call { args, dest, next, unwind, .. } => {
                live.or(&live_in[next.0 as usize]);
                let c = unwind.map(|u| &live_in[u.0 as usize]);
                old_value(&mut live, nst, dest, c, &mut mark);
                // The call itself unwinds with the destination's old value still there.
                if let Some(c) = c {
                    live.or(c);
                }
                for a in args {
                    if let Operand::Move(p) = a
                        && let Some(k) = rt(p)
                    {
                        mark(nst, p.local, Store::Clear, live.has(k));
                        live.del(k);
                    }
                }
            }
            Terminator::Panic(_) | Terminator::Return | Terminator::Unreachable | Terminator::MatchExhausted => {}
        }
        if !matches!(block.term, Terminator::Call { .. })
            && let Some(c) = edge(nst)
        {
            live.or(c);
        }
        for (si, st) in block.stmts.iter().enumerate().rev() {
            let si = si as u32;
            let c = edge(si);
            match &st.kind {
                StmtKind::Assign(dest, rv) => {
                    if idx.get(dest.local as usize).is_some_and(|&k| k != usize::MAX) && dest.proj.is_empty() {
                        old_value(&mut live, si, dest, c, &mut mark);
                    } else if let Some(c) = c {
                        live.or(c);
                    }
                    let mut moved = Vec::new();
                    rvalue_moves(rv, &mut moved);
                    for p in moved {
                        if let Some(k) = rt(p) {
                            mark(si, p.local, Store::Clear, live.has(k));
                            live.del(k);
                        }
                    }
                    if let Some(c) = c
                        && may_panic_before_moves(dest, rv)
                    {
                        live.or(c);
                    }
                }
                StmtKind::Drop(l) => match tv(bu, si, *l) {
                    Some(DYN) | Some(NONE) => {
                        if let Some(c) = c {
                            live.or(c);
                        }
                        if let Some(k) = rt(&Place::local(*l)) {
                            live.add(k);
                        }
                    }
                    Some(SET) => {
                        let k = rt(&Place::local(*l));
                        if let Some(c) = c {
                            live.or(c);
                        }
                        if let Some(k) = k {
                            mark(si, *l, Store::DropClear, live.has(k));
                            live.del(k);
                        }
                    }
                    Some(_) => {}
                    None => {
                        if let Some(c) = c {
                            live.or(c);
                        }
                    }
                },
                StmtKind::Print { .. } | StmtKind::StorageDead(_) | StmtKind::FakeRead(_) => {
                    if let Some(c) = c {
                        live.or(c);
                    }
                }
            }
        }
        live
    };
    let mut work: Vec<usize> = (0..nb).collect();
    let mut queued = vec![true; nb];
    while let Some(b) = work.pop() {
        queued[b] = false;
        let li = block_live(b, &live_in, &mut None);
        if li != live_in[b] {
            live_in[b] = li;
            for &p in &preds[b] {
                if !queued[p] {
                    queued[p] = true;
                    work.push(p);
                }
            }
        }
    }
    for b in 0..nb {
        block_live(b, &live_in, &mut Some(&mut dead));
    }
    let mut entry_live = vec![false; n];
    for (k, &l) in local_of.iter().enumerate() {
        entry_live[l as usize] = live_in[0].has(k);
    }
    (entry_live, dead)
}

/// Whether an assignment of `rv` to `dest` can panic before its moves are done: an indexed
/// destination's bounds check, or making the value (lowering's `may_unwind` rule for an
/// rvalue: moves, borrows and value building of plain operands with no indexed place cannot).
/// When false, the assignment cannot unwind at any point (it makes no call): cgen's R3 rule
/// relies on that stronger fact.
pub(crate) fn may_panic_before_moves(dest: &Place, rv: &Rvalue) -> bool {
    let indexed = |pl: &Place| pl.proj.iter().any(|p| matches!(p, Proj::Index(_)));
    let op_indexed = |o: &Operand| matches!(o, Operand::Copy(pl) | Operand::Move(pl) if indexed(pl));
    if indexed(dest) {
        return true;
    }
    match rv {
        Rvalue::Use(o) | Rvalue::Opt(_, Some(o)) | Rvalue::Res(_, _, o) => op_indexed(o),
        Rvalue::Ref(_, pl) | Rvalue::Discriminant(pl) | Rvalue::SliceOf(pl) | Rvalue::LenOf(pl) => indexed(pl),
        Rvalue::Aggregate(_, _, os) | Rvalue::TupleAgg(_, os) | Rvalue::ArrayAgg(_, os) => os.iter().any(op_indexed),
        Rvalue::Opt(_, None) | Rvalue::StaticRef(_) | Rvalue::FnValue(..) | Rvalue::ItemShim(..) => false,
        Rvalue::MakeDyn { ptr, .. } | Rvalue::DynUpcast { ptr, .. } | Rvalue::DynMeta { ptr, .. } | Rvalue::DynData { ptr, .. } => {
            op_indexed(ptr)
        }
        _ => true,
    }
}
