//! Safety contracts paired with Lust's typed program. Function identities and
//! local bindings come from the original checker. Only the additional safety
//! dimension is checked here; no source spelling substitutes for resolution.

use crate::ast::{Ast, Attr, AttrInput, BlockId, Expr, ExprId, ItemKind, Pat, PatId, Stmt, TyExpr, TyId};
use crate::check::collect::{FnLoc, LocalDefId, Program};
use crate::check::{Checked, PatRes, Res};
use crate::diag::{Diag, Span};
use crate::intern::Interner;
use crate::lexer::{Tok, TokenTree};
use crate::ty::{Ty, TyIntern, TyKind};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
pub struct NativeSafety {
    pub(crate) unsafe_blocks: HashMap<u32, Span>,
    pub(crate) unsafe_pointer_types: HashMap<u32, Span>,
    pub(crate) function_qualifiers: Vec<(Span, Span)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Contract {
    unsafe_call: bool,
    parameters: Vec<Option<Contract>>,
    result: Option<Box<Contract>>,
}

impl Contract {
    fn contains_unsafe(&self) -> bool {
        self.unsafe_call
            || self.parameters.iter().flatten().any(Self::contains_unsafe)
            || self.result.as_deref().is_some_and(Self::contains_unsafe)
    }
}

fn unsupported(span: Span, message: &str) -> Diag {
    Diag::new("L1000", span, message, "This native unsafe interface is not implemented yet.")
}

impl NativeSafety {
    pub fn has_extension(&self) -> bool {
        !self.unsafe_blocks.is_empty() || !self.unsafe_pointer_types.is_empty() || !self.function_qualifiers.is_empty()
    }

    pub fn validate_ast(&self, ast: &Ast, interner: &Interner) -> Result<(), Diag> {
        // A pointer qualifier must be represented at a concrete value boundary.
        // Aggregate, reference, alias and generic transport need richer type
        // metadata and are rejected rather than losing the qualifier.
        let mut direct = HashSet::new();
        for item in &ast.items {
            if let ItemKind::Fn(def) = &item.kind {
                for parameter in &def.params {
                    direct.insert(parameter.ty.0);
                }
                if let Some(ret) = def.ret {
                    direct.insert(ret.0);
                }
            }
        }
        for block in &ast.blocks {
            for statement in &block.stmts {
                if let Stmt::Let { ty: Some(ty), .. } = statement {
                    direct.insert(ty.0);
                }
            }
        }
        for expression in &ast.exprs {
            if let Expr::Cast { ty, .. } = expression {
                direct.insert(ty.0);
            }
        }
        for (&block, &span) in &self.unsafe_blocks {
            let expression = ast.exprs.iter().position(|expression| matches!(expression, Expr::Block(id) if id.0 == block));
            let owned = expression.is_some_and(|expression| {
                ast.items.iter().any(
                    |item| matches!(&item.kind, ItemKind::Fn(def) if def.exprs.0 <= expression as u32 && (expression as u32) < def.exprs.1),
                )
            });
            if !owned {
                return Err(unsupported(span, "unsafe expressions in signatures and item constants are not implemented"));
            }
        }
        for (&id, &span) in &self.unsafe_pointer_types {
            if !direct.contains(&id) {
                return Err(unsupported(
                    span,
                    "unsafe function pointers inside aggregate, reference, alias or nested signature types are not implemented",
                ));
            }
            let TyExpr::FnPtr { binder, qual, inputs, variadic, ret } = &ast.tys[id as usize] else {
                return Err(unsupported(span, "unsafe pointer annotation lost its native AST type"));
            };
            if !binder.is_empty() || *variadic || qual.is_async || qual.is_const || qual.abi.is_some() {
                return Err(unsupported(span, "qualified, variadic or higher-ranked unsafe function pointers are not implemented"));
            }
            let scalar = |ty: TyId| match ast.ty(ty) {
                TyExpr::Unit => true,
                TyExpr::Path(path) => {
                    path.len() == 1
                        && !path[0].has_args()
                        && matches!(
                            interner.get(path[0].name),
                            "bool"
                                | "char"
                                | "i8"
                                | "i16"
                                | "i32"
                                | "i64"
                                | "i128"
                                | "isize"
                                | "u8"
                                | "u16"
                                | "u32"
                                | "u64"
                                | "u128"
                                | "usize"
                                | "f32"
                                | "f64"
                        )
                }
                _ => false,
            };
            if inputs.iter().any(|p| !scalar(p.ty)) || ret.is_some_and(|ty| !scalar(ty)) {
                return Err(unsupported(span, "unsafe function pointer parameters and results currently require scalar types"));
            }
        }
        Ok(())
    }

    fn ast_contract(&self, ast: &Ast, ty: TyId) -> Option<Contract> {
        match ast.ty(ty) {
            TyExpr::FnPtr { qual, inputs, ret, .. } => Some(Contract {
                unsafe_call: qual.is_unsafe || self.unsafe_pointer_types.contains_key(&ty.0),
                parameters: inputs.iter().map(|p| self.ast_contract(ast, p.ty)).collect(),
                result: ret.and_then(|ty| self.ast_contract(ast, ty)).map(Box::new),
            }),
            TyExpr::Paren(ty) => self.ast_contract(ast, *ty),
            _ => None,
        }
    }

    fn function_contract(&self, program: &Program, fid: LocalDefId) -> Contract {
        let def = program.fn_def(fid);
        Contract {
            unsafe_call: def.qual.is_unsafe,
            parameters: def.params.iter().map(|p| self.ast_contract(&program.ast, p.ty)).collect(),
            result: def.ret.and_then(|ty| self.ast_contract(&program.ast, ty)).map(Box::new),
        }
    }

    /// The caller selects safety analysis once per program after collection.
    /// It must select it for unsafe imports even when this source has no new
    /// grammar, because calling such an import still requires an unsafe scope.
    pub fn check_body(&self, program: &Program, checked: &Checked<'_>, fid: LocalDefId) -> Vec<Diag> {
        let def = program.fn_def(fid);
        let Some(body) = def.body else { return Vec::new() };
        let result = def.ret.and_then(|ty| self.ast_contract(&program.ast, ty));
        let mut walk = Walk {
            safety: self,
            program,
            checked,
            locals: HashMap::new(),
            result,
            errors: Vec::new(),
            depth: 0,
            visited_blocks: HashSet::new(),
        };
        for (parameter, &local) in def.params.iter().zip(&checked.param_locals) {
            let contract = self.ast_contract(&program.ast, parameter.ty);
            walk.locals.insert(local, contract.clone());
            if let Some(pattern) = parameter.pat {
                walk.bind(pattern, contract);
            }
        }
        // FnDef.attrs includes the original parser's inherited allow records,
        // including duplicates of a function's own allow attributes. The root
        // item retains the actual written order needed for deny/allow precedence.
        let attrs = match program.fn_info(fid).loc {
            FnLoc::Item(index) => &program.ast.items[index as usize].attrs,
            _ => &def.attrs,
        };
        let level = walk.lint(attrs, Lint::Allow);
        if def.qual.is_unsafe && level.denied() {
            walk.unsafe_code(def.name_span);
        }
        // Rust's unsafe_op_in_unsafe_fn lint warns by default in edition 2024.
        // The original parser rejects deny/forbid levels for that unimplemented
        // lint, so its error behavior cannot be silently weakened here.
        let tail = walk.block(body, def.qual.is_unsafe, level);
        walk.expect(walk.result.clone().as_ref(), tail.as_ref(), program.ast.block(body).span);
        for index in def.exprs.0..def.exprs.1 {
            if let Expr::Block(block) = program.ast.expr(ExprId(index)) {
                if let Some(&span) = self.unsafe_blocks.get(&block.0) {
                    if !walk.visited_blocks.contains(&block.0) {
                        walk.error(span, "this type or constant context has no represented native unsafe scope");
                    }
                }
            }
        }
        walk.errors
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lint {
    Allow,
    Deny,
    Forbid,
}
impl Lint {
    fn denied(self) -> bool {
        self != Self::Allow
    }
}

struct Walk<'a, 'g> {
    safety: &'a NativeSafety,
    program: &'a Program,
    checked: &'a Checked<'g>,
    locals: HashMap<u32, Option<Contract>>,
    result: Option<Contract>,
    errors: Vec<Diag>,
    depth: usize,
    visited_blocks: HashSet<u32>,
}

impl Walk<'_, '_> {
    fn span(&self, id: ExprId) -> Span {
        self.program.ast.expr_span(id)
    }

    fn error(&mut self, span: Span, message: &str) {
        self.errors.push(unsupported(span, message));
    }

    fn unsafe_code(&mut self, span: Span) {
        self.errors.push(Diag::new(
            "L1003",
            span,
            "unsafe code is forbidden by the active unsafe_code lint",
            "Remove the unsafe code or change a non-forbidden lint level.",
        ));
    }

    fn lint(&mut self, attrs: &[Attr], mut level: Lint) -> Lint {
        for attr in attrs {
            let [name] = attr.path.as_slice() else { continue };
            let name = self.program.interner.get(name.name);
            if !matches!(name, "allow" | "warn" | "expect" | "deny" | "forbid") {
                continue;
            }
            let AttrInput::List(_, tokens) = &attr.input else { continue };
            let names_unsafe_code = tokens
                .split(|t| matches!(t, TokenTree::Token(t) if t.tok == Tok::Comma))
                .any(|path| matches!(path, [TokenTree::Token(t)] if matches!(t.tok, Tok::Ident(s) if self.program.interner.get(s) == "unsafe_code")));
            if !names_unsafe_code {
                continue;
            }
            let next = match name {
                "deny" => Lint::Deny,
                "forbid" => Lint::Forbid,
                _ => Lint::Allow,
            };
            if level == Lint::Forbid && next != Lint::Forbid {
                self.errors.push(Diag::new(
                    "L1003",
                    attr.span,
                    "unsafe_code is forbidden and cannot be overridden",
                    "Remove the conflicting lint override.",
                ));
            } else {
                level = next;
            }
        }
        level
    }

    fn typed_contract(&self, ty: Ty, depth: usize) -> Option<Contract> {
        if depth > 16 {
            return None;
        }
        match self.checked.tys.kind(ty) {
            TyKind::FnDef(def, _) => self.program.local_def(def.hash).map(|f| self.safety.function_contract(self.program, f)),
            TyKind::FnPtr(sig) => {
                let sig = self.checked.tys.sig(sig).value;
                Some(Contract {
                    unsafe_call: sig.is_unsafe,
                    parameters: self.checked.tys.tys_of(sig.inputs).iter().map(|&t| self.typed_contract(t, depth + 1)).collect(),
                    result: self.typed_contract(sig.output, depth + 1).map(Box::new),
                })
            }
            _ => None,
        }
    }

    fn resolved_contract(&self, expression: ExprId) -> Option<Contract> {
        match self.checked.res(expression) {
            Res::Fn(fid) => Some(self.safety.function_contract(self.program, fid)),
            Res::Local(local) => self.locals.get(&local).cloned().unwrap_or_else(|| self.typed_contract(self.checked.ty(expression), 0)),
            _ => self.typed_contract(self.checked.ty(expression), 0),
        }
    }

    fn expect(&mut self, expected: Option<&Contract>, actual: Option<&Contract>, span: Span) {
        match (expected, actual) {
            (Some(expected), Some(actual)) => {
                if !expected.unsafe_call && actual.unsafe_call {
                    self.errors.push(Diag::new(
                        "L1002",
                        span,
                        "an unsafe function cannot be converted to a safe function pointer",
                        "Preserve the unsafe function-pointer type.",
                    ));
                }
                if expected.parameters != actual.parameters || expected.result != actual.result {
                    if expected.contains_unsafe() || actual.contains_unsafe() {
                        self.error(span, "conversion between nested function contracts with different safety is not implemented");
                    }
                }
            }
            (None, Some(actual)) if actual.contains_unsafe() => self.error(span, "this use does not preserve the unsafe callable contract"),
            _ => {}
        }
    }

    fn bind(&mut self, pat: PatId, contract: Option<Contract>) {
        if let PatRes::Binding { local, .. } = self.checked.pat_res(pat) {
            self.locals.insert(local, contract.clone());
            if matches!(self.program.ast.pat(pat), Pat::Binding { .. }) {
                return;
            }
        }
        match self.program.ast.pat(pat) {
            Pat::Paren(inner) => self.bind(*inner, contract),
            Pat::At { inner, .. } => self.bind(*inner, contract),
            Pat::Wild => {}
            _ if contract.as_ref().is_some_and(Contract::contains_unsafe) => {
                self.error(self.program.ast.pat_span(pat), "destructuring an unsafe callable contract is not implemented")
            }
            _ => {}
        }
    }

    fn block(&mut self, id: BlockId, inherited: bool, level: Lint) -> Option<Contract> {
        self.visited_blocks.insert(id.0);
        let block = self.program.ast.block(id).clone();
        let level = self.lint(&block.inner_attrs, level);
        let introduced = self.safety.unsafe_blocks.get(&id.0).copied();
        if let Some(span) = introduced {
            if level.denied() {
                self.unsafe_code(span);
            }
        }
        let allowed = inherited || introduced.is_some();
        for statement in &block.stmts {
            match statement {
                Stmt::Let { pat, ty, init, els, attrs, span } => {
                    let statement_level = self.lint(attrs, level);
                    if let Some(ty) = ty {
                        self.ty(*ty, allowed, statement_level, 0);
                    }
                    let actual = init.and_then(|id| self.expr(id, allowed, statement_level));
                    let declared = ty.and_then(|id| self.safety.ast_contract(&self.program.ast, id));
                    let infer = ty.is_none() || ty.is_some_and(|id| matches!(self.program.ast.ty(id), TyExpr::Infer));
                    if !infer {
                        self.expect(declared.as_ref(), actual.as_ref(), *span);
                    }
                    self.bind(*pat, if infer { actual } else { declared });
                    if let Some(els) = els {
                        self.block(*els, allowed, statement_level);
                    }
                }
                Stmt::Expr { expr, attrs, .. } => {
                    let statement_level = self.lint(attrs, level);
                    self.expr(*expr, allowed, statement_level);
                }
                Stmt::Item(_) => {
                    self.error(block.span, "body-local items require an independent unsafe context and are not implemented on this path")
                }
                Stmt::Macro { .. } => self.error(block.span, "unexpanded statement macro in native safety analysis"),
            }
        }
        let tail_level = self.lint(&block.tail_attrs, level);
        block.tail.and_then(|tail| self.expr(tail, allowed, tail_level))
    }

    fn join(&mut self, a: Option<Contract>, b: Option<Contract>, span: Span) -> Option<Contract> {
        match (a, b) {
            (Some(mut a), Some(b)) if a.parameters == b.parameters && a.result == b.result => {
                a.unsafe_call |= b.unsafe_call;
                Some(a)
            }
            (Some(a), Some(b)) => {
                if a.contains_unsafe() || b.contains_unsafe() {
                    self.error(span, "joining distinct nested unsafe callable contracts is not implemented");
                }
                Some(a)
            }
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        }
    }

    fn ty(&mut self, id: TyId, allowed: bool, level: Lint, depth: usize) {
        if depth > 128 {
            self.error(self.program.ast.ty_span(id), "native unsafe type nesting limit exceeded");
            return;
        }
        match self.program.ast.ty(id).clone() {
            TyExpr::Array { inner, len } => {
                self.ty(inner, allowed, level, depth + 1);
                self.expr(len, allowed, level);
            }
            TyExpr::Tuple(types) => {
                for ty in types {
                    self.ty(ty, allowed, level, depth + 1);
                }
            }
            TyExpr::Ref { inner, .. } | TyExpr::Ptr { inner, .. } | TyExpr::Slice(inner) | TyExpr::Paren(inner) => {
                self.ty(inner, allowed, level, depth + 1)
            }
            TyExpr::FnPtr { inputs, ret, .. } => {
                for parameter in inputs {
                    self.ty(parameter.ty, allowed, level, depth + 1);
                }
                if let Some(ret) = ret {
                    self.ty(ret, allowed, level, depth + 1);
                }
            }
            _ => {}
        }
    }

    fn expr(&mut self, id: ExprId, allowed: bool, level: Lint) -> Option<Contract> {
        if id.0 < self.checked.expr_base || (id.0 - self.checked.expr_base) as usize >= self.checked.expr_ty.len() {
            self.error(self.span(id), "this unsafe expression is outside its checked native body");
            return None;
        }
        self.depth += 1;
        if self.depth > 1024 {
            self.error(self.span(id), "native unsafe expression nesting limit exceeded");
            self.depth -= 1;
            return None;
        }
        let expression = self.program.ast.expr(id).clone();
        let result = match expression {
            Expr::Path(_) | Expr::SelfValue => self.resolved_contract(id),
            Expr::Block(block) | Expr::UnsafeBlock(block) => self.block(block, allowed, level),
            Expr::Call { callee, args } => {
                let contract = self.expr(callee, allowed, level).or_else(|| self.resolved_contract(callee));
                if contract.as_ref().is_some_and(|c| c.unsafe_call) && !allowed {
                    self.errors.push(Diag::new(
                        "L1001",
                        self.span(id),
                        "calling this unsafe function requires an unsafe block",
                        "Wrap the call in an unsafe block after checking its contract.",
                    ));
                }
                for (index, argument) in args.iter().enumerate() {
                    let actual = self.expr(*argument, allowed, level);
                    let expected = contract.as_ref().and_then(|c| c.parameters.get(index)).and_then(Option::as_ref);
                    self.expect(expected, actual.as_ref(), self.span(*argument));
                }
                contract.and_then(|c| c.result.map(|r| *r))
            }
            Expr::MethodCall { recv, args, .. } => {
                let recv_contract = self.expr(recv, allowed, level);
                if recv_contract.as_ref().is_some_and(Contract::contains_unsafe) {
                    self.error(self.span(recv), "methods on unsafe callable values are not implemented");
                }
                for argument in args {
                    let actual = self.expr(argument, allowed, level);
                    self.expect(None, actual.as_ref(), self.span(argument));
                }
                self.typed_contract(self.checked.ty(id), 0)
            }
            Expr::Cast { expr, ty } => {
                self.ty(ty, allowed, level, 0);
                let actual = self.expr(expr, allowed, level);
                let expected = self.safety.ast_contract(&self.program.ast, ty);
                self.expect(expected.as_ref(), actual.as_ref(), self.span(id));
                expected
            }
            Expr::Assign { lhs, rhs } => {
                let expected = self.expr(lhs, allowed, level);
                let actual = self.expr(rhs, allowed, level);
                self.expect(expected.as_ref(), actual.as_ref(), self.span(rhs));
                None
            }
            Expr::If { cond, then, els } => {
                self.expr(cond, allowed, level);
                let first = self.block(then, allowed, level);
                let second = els.and_then(|e| self.expr(e, allowed, level));
                if els.is_some() { self.join(first, second, self.span(id)) } else { None }
            }
            Expr::Match { scrut, arms } => {
                let scrutinee = self.expr(scrut, allowed, level);
                if scrutinee.as_ref().is_some_and(Contract::contains_unsafe) {
                    self.error(self.span(scrut), "matching an unsafe callable value is not implemented");
                }
                let mut result = None;
                for arm in arms {
                    let arm_level = self.lint(&arm.attrs, level);
                    if let Some(guard) = arm.guard {
                        self.expr(guard, allowed, arm_level);
                    }
                    let next = self.expr(arm.body, allowed, arm_level);
                    result = self.join(result, next, self.span(id));
                }
                result
            }
            Expr::Closure { params, ret, body, .. } => {
                for parameter in params {
                    if let Some(ty) = parameter.ty {
                        self.ty(ty, allowed, level, 0);
                    }
                    let contract = parameter.ty.and_then(|t| self.safety.ast_contract(&self.program.ast, t));
                    self.bind(parameter.pat, contract);
                }
                let expected = ret.and_then(|t| self.safety.ast_contract(&self.program.ast, t));
                if let Some(ty) = ret {
                    self.ty(ty, allowed, level, 0);
                }
                let outer_result = std::mem::replace(&mut self.result, expected.clone());
                let actual = self.expr(body, allowed, level);
                self.result = outer_result;
                // Inferred closure return types need dedicated callable-type
                // transport before they may return an unsafe pointer or item.
                self.expect(expected.as_ref(), actual.as_ref(), self.span(body));
                None
            }
            Expr::Return(value) => {
                let actual = value.and_then(|value| self.expr(value, allowed, level));
                self.expect(self.result.clone().as_ref(), actual.as_ref(), self.span(id));
                None
            }
            Expr::Break { value, .. } => {
                let actual = value.and_then(|value| self.expr(value, allowed, level));
                if actual.as_ref().is_some_and(Contract::contains_unsafe) {
                    self.error(self.span(id), "unsafe callable break values are not implemented");
                }
                None
            }
            Expr::While { cond, body, .. } => {
                self.expr(cond, allowed, level);
                self.block(body, allowed, level);
                None
            }
            Expr::Loop { body, .. } => {
                self.block(body, allowed, level);
                None
            }
            Expr::LabeledBlock { body, .. } => self.block(body, allowed, level),
            Expr::For { iter, body, .. } => {
                let actual = self.expr(iter, allowed, level);
                if actual.as_ref().is_some_and(Contract::contains_unsafe) {
                    self.error(self.span(iter), "unsafe callable iteration is not implemented");
                }
                self.block(body, allowed, level);
                None
            }
            Expr::Tuple(values) | Expr::Array(values) | Expr::Macro { args: values, .. } => {
                for value in values {
                    let actual = self.expr(value, allowed, level);
                    if actual.as_ref().is_some_and(Contract::contains_unsafe) {
                        self.error(self.span(value), "unsafe callable aggregate or macro argument transport is not implemented");
                    }
                }
                None
            }
            Expr::StructLit { fields, update, .. } => {
                for (_, _, value) in fields {
                    let actual = self.expr(value, allowed, level);
                    if actual.as_ref().is_some_and(Contract::contains_unsafe) {
                        self.error(self.span(value), "unsafe callable struct fields are not implemented");
                    }
                }
                if let Some(value) = update {
                    self.expr(value, allowed, level);
                }
                None
            }
            Expr::Ref { expr, .. } | Expr::Deref(expr) => {
                let actual = self.expr(expr, allowed, level);
                if actual.as_ref().is_some_and(Contract::contains_unsafe) {
                    self.error(self.span(id), "references to unsafe callable values are not implemented");
                }
                self.typed_contract(self.checked.ty(id), 0)
            }
            Expr::Field { base, .. } | Expr::TupleField { base, .. } => {
                self.expr(base, allowed, level);
                self.typed_contract(self.checked.ty(id), 0)
            }
            Expr::Index { base, index } => {
                self.expr(base, allowed, level);
                self.expr(index, allowed, level);
                self.typed_contract(self.checked.ty(id), 0)
            }
            Expr::Binary { lhs, rhs, .. } | Expr::AssignOp { lhs, rhs, .. } => {
                self.expr(lhs, allowed, level);
                self.expr(rhs, allowed, level);
                None
            }
            Expr::Unary { expr, .. } | Expr::Try(expr) | Expr::Await(expr) => {
                self.expr(expr, allowed, level);
                self.typed_contract(self.checked.ty(id), 0)
            }
            Expr::ArrayRepeat { value, len } => {
                let actual = self.expr(value, allowed, level);
                if actual.as_ref().is_some_and(Contract::contains_unsafe) {
                    self.error(self.span(value), "unsafe callable arrays are not implemented");
                }
                self.expr(len, allowed, level);
                None
            }
            Expr::Range { lo, hi, .. } => {
                if let Some(e) = lo {
                    self.expr(e, allowed, level);
                }
                if let Some(e) = hi {
                    self.expr(e, allowed, level);
                }
                None
            }
            Expr::Let { pat, scrut } => {
                let actual = self.expr(scrut, allowed, level);
                self.bind(pat, actual);
                None
            }
            Expr::ConstBlock(block) | Expr::AsyncBlock { body: block, .. } => {
                let result = self.block(block, allowed, level);
                if result.as_ref().is_some_and(Contract::contains_unsafe) {
                    self.error(self.span(id), "unsafe callable const or async result transport is not implemented");
                }
                None
            }
            _ => self.typed_contract(self.checked.ty(id), 0),
        };
        self.depth -= 1;
        result
    }
}
