//! Port of Go `ls/languageservice.go`.

use crate::ls::prelude::*;

// Go: ls/languageservice.go:16 LanguageService
// PORT: plan contract C4. Go `*compiler.Program` is `Rc<compiler::NewProgram>`:
// the language service keeps its program alive. Go `Host` is `Rc<dyn Host>`, Go
// `*lsconv.Converters` is shared (`Rc`). All methods take `&self`, so the
// mapper cache is a `RefCell`. The fields are `pub` because other files of
// package `ls` read them (Go same-package access).
// PORT: `P` is the program. It is `compiler::NewProgram` except on a search
// thread of a cross-project request (`SearchView`, `program_view.rs`). The
// methods that the search runs are in `impl<P: ProgramView>
// LanguageService<P>` blocks; all other methods are for the default `P`
// only.

// Go: ls/languageservice.go:25 NewLanguageService
// PORT: Go returns `*LanguageService`; the caller owns the value here.





// PORT: Go `*LanguageService` implements `sourcemap.Host` through its
// `UseCaseSensitiveFileNames`, `GetECMALineInfo` and `ReadFile` methods
// (`GetDocumentPositionMapper` passes `l`). The inherent methods above win in
// method syntax, so these calls do not recurse.
