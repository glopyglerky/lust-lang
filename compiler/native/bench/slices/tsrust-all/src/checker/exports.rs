use crate::prelude::*;

// Port of checker/exports.go (all): the exported Checker API that the
// language service calls.
//
// PORT: names. When the unexported Go twin has the same snake name, the
// exported method ends in `_exported` (PORTING.md). The names follow
// `checker-api-tools/names_exports_services.tsv` column 5.
// PORT: Go package-level functions in this file (`IsTypeUsableAsPropertyName`,
// `GetPropertyNameFromType`, `GetDeclarationModifierFlagsFromSymbol`,
// `IsTupleType`) read type or symbol data, so they are Checker methods.
// PORT: a Go `[]*T` result is a `Vec<T>`. Where the twin returns a
// `SharedList`, the wrapper copies it into a `Vec`.

