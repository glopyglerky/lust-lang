// `std::process::ExitCode` in body checking (W-STD-A phase 3): the
// `SUCCESS` and `FAILURE` consts and `ExitCode::from(u8)`. The type
// is a builtin Adt (ty/mod.rs `BuiltinDefs::exit_code`) that cgen
// keeps as a `uint8_t`; `main` may return it (collect.rs).
use super::*;

impl<'p> Ctx<'p> {
    /// True when `segs` is `ExitCode::item`: the bare type name when
    /// no user type takes it, or the canonical std path.
    fn exit_code_path(&self, segs: &[ast::PathSeg]) -> bool {
        let n = segs.len();
        if n < 2 || self.name(segs[n - 2].name) != "ExitCode" {
            return false;
        }
        if n == 2 {
            return !self.p.type_names.contains_key(&segs[0].name);
        }
        let path: Vec<&str> = segs[..n - 1].iter().map(|s| self.name(s.name)).collect();
        crate::ty::lang::std_path_kind(&path) == Some(crate::ty::lang::StdPathKind::Type)
    }

    /// `ExitCode::SUCCESS` and `ExitCode::FAILURE`: const values that
    /// lower through `ExitCode::from` (lower.rs `const_into`).
    pub(super) fn exit_code_const(&mut self, e: ExprId, segs: &[ast::PathSeg]) -> Option<Ty> {
        if !self.exit_code_path(segs) {
            return None;
        }
        let v = match self.name(segs[segs.len() - 1].name) {
            "SUCCESS" => 0,
            "FAILURE" => 1,
            _ => return None,
        };
        let i = self.ei(e);
        self.out.const_exprs[i] = Some(crate::ty::consteval::ConstValue::Int(v));
        Some(self.tys.exit_code_ty())
    }

    /// `ExitCode::from`, the one associated fn lust has for it. None
    /// when `ty_name` is not std's `ExitCode` (the caller goes on).
    pub(super) fn exit_code_assoc(&mut self, ty_name: Sym, item: Sym, std_ty: bool, span: Span) -> Option<Option<Res>> {
        if self.name(ty_name) != "ExitCode" || (!std_ty && self.p.type_names.contains_key(&ty_name)) {
            return None;
        }
        if item == self.p.known.from {
            return Some(Some(Res::Builtin(Builtin::ExitCodeFrom)));
        }
        self.err(
            "L0104",
            span,
            format!("no function `ExitCode::{}` in lust yet", self.name(item)),
            "Lust supports ExitCode::SUCCESS, ExitCode::FAILURE, and ExitCode::from.",
        );
        Some(None)
    }
}
