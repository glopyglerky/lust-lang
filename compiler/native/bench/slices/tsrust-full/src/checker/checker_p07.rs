//! Port of Go `checker/checker.go` lines 5715-6625 (checker-07):
//! `hasExportedMembersOfKind` through `getIterationTypesOfMethod`.
//!
//! PORT: cross-file choices this file makes (the contract does not settle them):
//! - Go stored `func` fields on `Checker` (`getGlobalIterableType`,
//!   `getGlobalDisposableType`, `resolveName`, ...) are
//!   `Rc<dyn Fn(&mut Checker, ...) -> ...>` fields and are called as
//!   `(self.field.clone())(self, ...)`.
//! - `c.syncIterationTypesResolver` / `c.asyncIterationTypesResolver` are
//!   `Rc<IterationTypesResolver>`. Functions that take a Go
//!   `*IterationTypesResolver` take `&IterationTypesResolver`.
//! - Nilable `*diagnostics.Message` parameters are `Option<&'static Message>`.
//! - Go `diagnosticOutput *[]*ast.Diagnostic` / `candidatesOutArray
//!   *[]*Signature` are `Option<&mut Vec<Diagnostic>>` /
//!   `Option<&mut Vec<SignatureId>>`.
//! - Go `c.addDeferredDiagnostic(func())` takes `Rc<dyn Fn(&mut Checker)>`.

use crate::prelude::*;


// Go: checker/checker.go:5921 isNotOverload





