// `thread_local!` statics and the `std::thread::LocalKey` API (ARCH 7.3).
//
// The macro (expand/builtin.rs) turns `static NAME: T = init;` into a
// static marked `#[thread_local!]` whose value is the path to an init fn
// `fn NAME#init() -> T { init }` (`#[thread_local!const]` for the
// `const { e }` form, whose block is the init fn body). Collect types the
// static as `LocalKey<T>` with every region of `T` `'static` (its C value
// is the key's slot word, 1, 2, ..) and records the init fn here. Method
// calls on the key type as builtin methods; lowering adds the init fn and
// the key form as operands (lower/tls.rs), and the expansion after borrowck
// turns each call into the runtime access plus plain IR (closure calls,
// Cell and RefCell builtins).
use super::*;
use crate::ast::{Attr, Item, ItemKind};

/// What collect knows about a `thread_local!` static (`StaticItem::tls`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TlsKey {
    /// The init fn the expansion made: the lazy initializer, or the
    /// `const { .. }` block's value.
    pub init: LocalDefId,
    /// The `const { .. }` form: the value counts as initialized from
    /// the start, so `set` never skips it (Rust's `EagerStorage`).
    pub eager: bool,
}

/// The `thread_local!` form of a static's attributes: None for a plain
/// static, else whether it is the `const { .. }` form.
pub fn key_form(attrs: &[Attr], interner: &Interner) -> Option<bool> {
    attrs.iter().find_map(|a| match a.path.as_slice() {
        [seg] => match interner.get(seg.name) {
            crate::parser::TLS_ATTR => Some(false),
            crate::parser::TLS_CONST_ATTR => Some(true),
            _ => None,
        },
        _ => None,
    })
}

/// The init fn of a `thread_local!` static: its value is a path to a fn
/// of the same block (`stmt_idx`, a body item) or of the crate's flat
/// fn table. None only for a static the macro did not make.
pub fn init_fn(p: &Program, value: ast::ExprId, stmt_idx: Option<u32>) -> Option<LocalDefId> {
    let ast::Expr::Path(segs) = p.ast.expr(value) else { return None };
    let [seg] = segs.as_slice() else { return None };
    let Some(sidx) = stmt_idx else { return p.fn_names.get(&seg.name).copied() };
    // The expansion puts the init fn right after its static, in the same
    // block; the first fn of that name after the static is it.
    let fn_named = |it: &Item| matches!(&it.kind, ItemKind::Fn(d) if d.name == seg.name);
    let at = p.ast.stmt_items.iter().enumerate().skip(sidx as usize + 1).find(|(_, it)| fn_named(it))?.0;
    match p.body_items.get(&(at as u32)) {
        Some(BodyItem::Fn(f)) => Some(*f),
        _ => None,
    }
}

/// The `LocalKey` methods of Rust 1.99. `with` and `try_with` take any
/// value type; the rest are on `LocalKey<Cell<T>>` and
/// `LocalKey<RefCell<T>>` only.
const KEY_METHODS: &[&str] = &["with", "try_with", "with_borrow", "with_borrow_mut", "set", "get", "take", "replace"];

impl<'p> Ctx<'p> {
    /// The `thread_local!` static a method receiver names directly
    /// (`KEY.with(..)`), by static index. A key reached any other way
    /// (a `&'static LocalKey<T>` value) is not supported yet: lowering
    /// needs the static to find its init fn.
    pub(super) fn tls_static_of(&self, recv: ExprId) -> Option<u32> {
        match self.out.expr_res[self.ei(recv)] {
            Res::Static(idx) if self.p.statics.get(idx as usize).is_some_and(|s| s.tls.is_some()) => Some(idx),
            _ => None,
        }
    }

    /// Methods on a `LocalKey<T>` receiver (after autoderef). None when
    /// the receiver is no key or the name is no `LocalKey` method (a
    /// user trait method then gets its chance).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn tls_method(&mut self, e: ExprId, recv: ExprId, base: Ty, derefs: u8, name: Sym, span: Span, args: &[ExprId]) -> Option<Ty> {
        let t = self.tys.local_key_elem(base)?;
        let n = self.name(name).to_string();
        if !KEY_METHODS.contains(&n.as_str()) {
            return None;
        }
        if derefs != 0 || self.tls_static_of(recv).is_none() {
            self.err(
                "L0640",
                span,
                format!("`LocalKey::{n}` on a key that is not a `thread_local!` static by name is not supported yet"),
                "Call the method on the static itself, as in `KEY.with(|v| ..)`.",
            );
            return Some(self.tls_bail(args));
        }
        let t = self.resolve(t);
        use Builtin as B;
        if let Some(found) = self.tls_path_callable(&n, t, args, span) {
            let (b, ret) = found;
            self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
            return Some(ret);
        }
        let (b, params, ret) = match n.as_str() {
            "with" | "try_with" => {
                let r = self.new_var(false);
                let rt = self.tys.mk_ref(false, t);
                let f = self.fn_once_param(&[rt], r, span);
                if n == "with" {
                    (B::TlsWith, vec![f], r)
                } else {
                    let ae = self.tys.access_error_ty();
                    (B::TlsTryWith, vec![f], self.tys.mk_result(r, ae))
                }
            }
            _ => {
                // `LocalKey<Cell<U>>` and `LocalKey<RefCell<U>>` methods.
                let cell = self.tys.cell_elem(t);
                let refcell = self.tys.refcell_elem(t);
                let found = match (n.as_str(), cell, refcell) {
                    ("with_borrow" | "with_borrow_mut", _, Some(u)) => {
                        let r = self.new_var(false);
                        let ut = self.tys.mk_ref(n == "with_borrow_mut", u);
                        let f = self.fn_once_param(&[ut], r, span);
                        Some(if n == "with_borrow" { (B::TlsWithBorrow, vec![f], r) } else { (B::TlsWithBorrowMut, vec![f], r) })
                    }
                    ("set", Some(u), _) | ("set", _, Some(u)) => Some((B::TlsSet, vec![u], UNIT)),
                    ("replace", Some(u), _) | ("replace", _, Some(u)) => Some((B::TlsReplace, vec![u], u)),
                    ("get", Some(u), _) => {
                        self.deferred_copy.push((u, span));
                        Some((B::TlsGet, vec![], u))
                    }
                    ("take", Some(u), _) | ("take", _, Some(u)) => {
                        self.deferred_default.push((u, span));
                        Some((B::TlsTake, vec![], u))
                    }
                    _ => None,
                };
                let Some(found) = found else {
                    let tn = self.tn(base);
                    let need = if n.starts_with("with_borrow") { "a `RefCell` value" } else { "a `Cell` or `RefCell` value (`get`: a `Cell`)" };
                    self.err("L0104", span, format!("no method `{n}` on type `{tn}`"), format!("`LocalKey::{n}` needs {need}."));
                    return Some(self.tls_bail(args));
                };
                found
            }
        };
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
        self.check_args(&params, args, span);
        Some(ret)
    }

    /// `KEY.with(Cell::get)`, `KEY.try_with(Cell::get)`, and
    /// `KEY.with(Rc::clone)`: a std fn path as the callable (lust has no
    /// fn item type for builtin methods). The path must name std's type:
    /// a std path, or a bare name no user type shadows (the resolver
    /// checks its import). Lowering gives the path no operand.
    fn tls_path_callable(&mut self, n: &str, t: Ty, args: &[ExprId], span: Span) -> Option<(Builtin, Ty)> {
        let [a] = args else { return None };
        let ast::Expr::Path(segs) = self.p.ast.expr(*a) else { return None };
        let names: Vec<&str> = segs.iter().map(|s| self.p.interner.get(s.name)).collect();
        let (ty, item, prefix) = match names.as_slice() {
            [ty, item] => (*ty, *item, None),
            [k, m, ty, item] => (*ty, *item, Some((*k, *m))),
            _ => return None,
        };
        if segs.iter().any(|s| !s.type_args().is_empty()) {
            return None;
        }
        let ty_sym = segs[segs.len() - 2].name;
        let std_named = |module: &str| match prefix {
            None => !self.p.type_names.contains_key(&ty_sym),
            Some((k, m)) => matches!(k, "std" | "core" | "alloc") && m == module,
        };
        use Builtin as B;
        let found = match (n, ty, item) {
            ("with" | "try_with", "Cell", "get") if std_named("cell") => {
                let u = self.tys.cell_elem(t)?;
                self.deferred_copy.push((u, span));
                if n == "with" {
                    (B::TlsGet, u)
                } else {
                    let ae = self.tys.access_error_ty();
                    (B::TlsTryGet, self.tys.mk_result(u, ae))
                }
            }
            ("with", "Rc", "clone") if std_named("rc") => {
                self.tys.rc_elem(t)?;
                (B::TlsCloneRc, t)
            }
            _ => return None,
        };
        let i = self.ei(*a);
        self.out.expr_ty[i] = UNIT;
        Some(found)
    }

    /// The `const { .. }` form of `thread_local!`: rustc makes its block
    /// a `const` item, so the block must be a const initializer. Lust
    /// evaluates the value at run time (the init fn) like a lazy one,
    /// which is the same value: const evaluation has no side effects.
    /// So this proves the const part only: the const constructors of
    /// builtin types (`Cell::new`, `RefCell::new`, `Vec::new`,
    /// `String::new`) and aggregates of const parts are const by their
    /// shape; every other part must pass the bounded const evaluator, as
    /// an inline `const` block does. Runs after the init fn's body check.
    pub(super) fn tls_const_init_check(&mut self) {
        let fid = self.fid;
        if !self.p.statics.iter().any(|s| s.tls.is_some_and(|k| k.eager && k.init == fid)) {
            return;
        }
        // The init fn body is `{ { .. } }`: its tail is the const block.
        let def = self.p.fn_def(fid);
        if let Some(b) = def.body
            && let Some(t) = self.p.ast.block(b).tail
        {
            self.tls_const_expr(t);
        }
    }

    /// Proves one part of a const initializer (see `tls_const_init_check`).
    /// A block with statements goes to the evaluator whole.
    fn tls_const_expr(&mut self, e: ExprId) {
        use Builtin as B;
        let p = self.p;
        let res = self.out.expr_res[self.ei(e)];
        let parts: Vec<ExprId> = match p.ast.expr(e) {
            ast::Expr::Call { args, .. }
                if matches!(
                    res,
                    Res::Builtin(B::CellNew | B::RefCellNew | B::VecNew | B::StringNew) | Res::OptSome | Res::ResOk | Res::ResErr | Res::Variant(..)
                ) =>
            {
                args.clone()
            }
            ast::Expr::StructLit { fields, update: None, .. } => fields.iter().map(|f| f.2).collect(),
            ast::Expr::Tuple(es) | ast::Expr::Array(es) => es.clone(),
            ast::Expr::Block(b) if p.ast.block(*b).stmts.is_empty() && p.ast.block(*b).tail.is_some() => vec![p.ast.block(*b).tail.unwrap()],
            _ => {
                let t = self.out.expr_ty[self.ei(e)];
                let mut ccx = CheckCx { cx: self };
                if let Err(d) = crate::ty::consteval::eval(&mut ccx, e, Some(t)) {
                    ccx.cx.errs.push(d);
                }
                return;
            }
        };
        for a in parts {
            self.tls_const_expr(a);
        }
    }

    /// Checks the arguments of a rejected call (so their own errors
    /// still report) and answers the error type.
    fn tls_bail(&mut self, args: &[ExprId]) -> Ty {
        for &a in args {
            self.expr(a, None);
        }
        ERROR
    }
}
