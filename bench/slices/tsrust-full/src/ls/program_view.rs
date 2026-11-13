//! The program reads of the cross-project search: the Go
//! `*compiler.Program` methods that `provideSymbolsAndEntries`,
//! `forEachOriginalDefinitionLocation` and the `symbolAndEntriesTo*`
//! functions of references, implementations and rename call.
//!
//! PORT: Go runs the search of each project on its own goroutine, and every
//! goroutine reads the shared `*compiler.Program`. `NewProgram` is not
//! `Sync` (it holds `Rc` and `RefCell` values), so a search thread
//! (`search_thread.rs`) cannot read it. The search code is generic over
//! `ProgramView` instead. `NewProgram` implements it on the dispatch
//! thread. `SearchView` implements it on a search thread with a copy of the
//! data it reads, and asks the dispatch thread for the rare reads that the
//! copy does not hold. `LanguageService<P>` has the type parameter
//! `P = NewProgram`, so code outside the search path does not change. The
//! search code takes the program as `&P`.

use crate::ls::prelude::*;


