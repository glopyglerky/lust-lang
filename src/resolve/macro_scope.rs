//! The textual `macro_rules!` scope of one file (ARCH 6.1), for imports.
//!
//! A `macro_rules!` definition is not in its module's namespace. rustc
//! puts a macro there only through a `use` of it (the re-export idiom
//! `macro_rules! m { .. } pub(crate) use m;`) or, at the crate root,
//! through `#[macro_export]`. The path of a one-segment `use m;` reads
//! the definitions textually in scope at the `use` first: one before it
//! in the same module, one an enclosing module defines before the
//! `mod` item, or one a `#[macro_use] mod` before it exports. The
//! resolve worklist builds one scope per file from its textual macro
//! environment, and both the macro re-export fixpoint and the import
//! fixpoint query it.

use crate::intern::Sym;

/// One textual join point: from byte offset `lo` on, the macros
/// `macros` are in scope in the inline container `container` and in
/// the inline modules inside it. `skip_inside` is the module of a
/// `#[macro_use] mod m { .. }` join, where the join does not apply
/// (inside `m` its own definitions apply, in order).
#[derive(Clone, Debug)]
pub struct ScopeEvent<T> {
    pub lo: u32,
    pub container: Vec<Sym>,
    pub skip_inside: Option<Vec<Sym>>,
    pub macros: Vec<(Sym, T)>,
}

/// The textual macro scope of one file: the macros the whole file
/// inherits from its parent modules, then its own join points in
/// offset order. `T` names a macro: an owned source index in the
/// re-export fixpoint, a definition (None for a dependency's macro) in
/// the import fixpoint.
#[derive(Clone, Debug)]
pub struct FileMacroScope<T> {
    pub inherited: Vec<(Sym, T)>,
    pub events: Vec<ScopeEvent<T>>,
}

impl<T: Copy> FileMacroScope<T> {
    /// The macro named `name` in textual scope at offset `lo` in the
    /// inline container `container` (the path of inline `mod`s from the
    /// file's own module). A later definition shadows an earlier one.
    pub fn at(&self, container: &[Sym], lo: u32, name: Sym) -> Option<T> {
        let mut found = self.inherited.iter().rev().find(|(n, _)| *n == name).map(|(_, m)| *m);
        for ev in &self.events {
            if ev.lo >= lo {
                break;
            }
            if !container.starts_with(&ev.container) || ev.skip_inside.as_ref().is_some_and(|inside| container.starts_with(inside)) {
                continue;
            }
            if let Some((_, m)) = ev.macros.iter().rev().find(|(n, _)| *n == name) {
                found = Some(*m);
            }
        }
        found
    }

    /// The same scope with each macro mapped (owned index to definition).
    pub fn map<U>(&self, f: impl Fn(T) -> U) -> FileMacroScope<U> {
        let conv = |v: &[(Sym, T)]| v.iter().map(|&(n, m)| (n, f(m))).collect();
        FileMacroScope {
            inherited: conv(&self.inherited),
            events: self
                .events
                .iter()
                .map(|e| ScopeEvent { lo: e.lo, container: e.container.clone(), skip_inside: e.skip_inside.clone(), macros: conv(&e.macros) })
                .collect(),
        }
    }
}
