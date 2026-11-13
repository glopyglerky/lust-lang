//! Go `internal/project/project.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go `*Project` is
//! `Rc<RefCell<Project>>` (the dirty maps change it through the pointer).
//! Go `*compiler.Program` is `Rc<compiler::NewProgram>` (nil is
//! `None`). Go `*tsoptions.ParsedCommandLine` is
//! `Option<Rc<tsoptions::ParsedCommandLine>>`; Go pointer equality is
//! `Rc::ptr_eq`.

use crate::project::prelude::*;

use crate::contentmapper;
use crate::frontend::core_ext::{ProjectReference, TypeAcquisition};
use crate::frontend::module;
use crate::program::ls_program;
use std::cell::Cell;

// Go: project/project.go:25 inferredProjectName
pub const INFERRED_PROJECT_NAME: &str = "/dev/null/inferred"; // lowercase so toPath is a no-op regardless of settings
// Go: project/project.go:26 syntheticProjectPrefix (ts#64204)
pub const SYNTHETIC_PROJECT_PREFIX: &str = "/dev/null/synthetic/";
// Go: project/project.go:27 hr
pub const HR: &str = "-----------------------------------------------";

// Go: project/project.go:31 ID (ts#64319)
// PORT: Go `type ID string`. The derived `Ord` is Go's string order
// (`cmp.Compare`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ID(pub String);

// Go: project/project.go:34 ConfiguredProjectID (ts#64319)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConfiguredProjectID(pub tspath::Path);

impl ConfiguredProjectID {
    // Go: project/project.go:36 ConfiguredProjectID.Path
    pub fn path(&self) -> tspath::Path {
        self.0.clone()
    }

    // Go: project/project.go:54 ConfiguredProjectID.AsID
    pub fn as_id(&self) -> ID {
        ID(self.0.0.clone())
    }
}

// Go: project/project.go:40 InferredProjectID (ts#64319)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InferredProjectID(pub String);

// Go: project/project.go:42 inferredProjectID
// PORT: Go `const inferredProjectID InferredProjectID = inferredProjectName`.
pub fn inferred_project_id() -> InferredProjectID {
    InferredProjectID(INFERRED_PROJECT_NAME.to_string())
}

impl InferredProjectID {
    // Go: project/project.go:55 InferredProjectID.AsID
    pub fn as_id(&self) -> ID {
        ID(self.0.clone())
    }
}

// Go: project/project.go:44 SyntheticProjectID (ts#64319)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SyntheticProjectID(pub String);

// Go: project/project.go:46 NewSyntheticProjectID (ts#64319)
pub fn new_synthetic_project_id(id: i32) -> SyntheticProjectID {
    if id <= 0 {
        crate::core::go_panic(format!("invalid synthetic project ID: {id}"));
    }
    SyntheticProjectID(format!("{SYNTHETIC_PROJECT_PREFIX}{id}"))
}

impl SyntheticProjectID {
    // Go: project/project.go:56 SyntheticProjectID.AsID
    pub fn as_id(&self) -> ID {
        ID(self.0.clone())
    }
}

/// Go `%s` of an ID type is its string.
impl std::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for ConfiguredProjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.0)
    }
}

impl std::fmt::Display for SyntheticProjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl ID {
    // Go: project/project.go:53 ID.String
    pub fn string(&self) -> String {
        self.0.clone()
    }

    // Go: project/project.go:58 ID.Configured

    // Go: project/project.go:76 ID.Inferred
    pub fn inferred(&self) -> (InferredProjectID, bool) {
        (
            inferred_project_id(),
            *self == inferred_project_id().as_id(),
        )
    }

    // Go: project/project.go:80 ID.Synthetic

    /// Go: an `ID` passed as an `autoimport.ProjectID` (the Go interface holds
    /// the `ID`; the Rust type holds its string).
    // PORT: not in Go.
    pub fn as_auto_import_project_id(&self) -> autoimport::ProjectID {
        autoimport::ProjectID(self.0.clone())
    }
}

// PORT: Go passes an `ID` as an `ata.ProjectID` (`fmt.Stringer`).
impl ata::ProjectID for ID {
    fn string(&self) -> String {
        self.0.clone()
    }
}

// Go: project/project.go:62 ParseConfiguredProjectID (ts#64319)

// Go: project/project.go:84 SyntheticProjectID.UnmarshalJSONFrom (ts#64319)
// PORT: the JSON impls of the project ID types are in src/api/proto.rs, with
// the other API JSON impls.

// Go: project/project.go:97 ParseSyntheticProjectID (ts#64319)
// PORT: Go `strconv.Atoi` accepts a leading sign; `str::parse::<i32>` does
// too.

// Go: project/project.go:111 Kind
// PORT: Go `type Kind int` with iota consts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Kind(pub i32);

impl Kind {
    // Go: project/project.go:114 KindInferred
    pub const INFERRED: Kind = Kind(0);
    // Go: project/project.go:115 KindConfigured
    pub const CONFIGURED: Kind = Kind(1);
    // Go: project/project.go:51 KindSynthetic (ts#64204)
    pub const SYNTHETIC: Kind = Kind(2);
}

// Go: project/project.go:119 ProgramUpdateKind
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProgramUpdateKind(pub i32);

impl ProgramUpdateKind {
    // Go: project/project.go:122 ProgramUpdateKindNone
    pub const NONE: ProgramUpdateKind = ProgramUpdateKind(0);
    // Go: project/project.go:123 ProgramUpdateKindCloned
    pub const CLONED: ProgramUpdateKind = ProgramUpdateKind(1);
    // Go: project/project.go:124 ProgramUpdateKindSameFileNames
    pub const SAME_FILE_NAMES: ProgramUpdateKind = ProgramUpdateKind(2);
    // Go: project/project.go:125 ProgramUpdateKindNewFiles
    pub const NEW_FILES: ProgramUpdateKind = ProgramUpdateKind(3);
}

// Go: project/project.go:128 PendingReload
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PendingReload(pub i32);

impl PendingReload {
    // Go: project/project.go:131 PendingReloadNone
    pub const NONE: PendingReload = PendingReload(0);
    // Go: project/project.go:132 PendingReloadFileNames
    pub const FILE_NAMES: PendingReload = PendingReload(1);
    // Go: project/project.go:133 PendingReloadFull
    pub const FULL: PendingReload = PendingReload(2);
}

// Go: project/project.go:138 Project
// Project represents a TypeScript project.
// If changing struct fields, also update the Clone method.
// PORT: `commandLineWithTypingsFiles` and its `sync.Once` are written by
// `getCommandLineWithTypingsFiles`, which `CreateProgram` calls; they are a
// `RefCell` and a `Cell<bool>` so both take `&self`. Go
// `*collections.Set[tspath.Path]` is `Option<Rc<FxHashSet<..>>>` (Go
// replaces it with a clone before adding). Go
// `*collections.SyncSet[tspath.Path]` (the program files watch input) is
// `Option<Rc<RefCell<FxHashSet<..>>>>`, the sourceFS seen-files set.

// Go: project/project.go:180 NewConfiguredProject
// PORT: Go `*ProjectCollectionBuilder` is only read here, so it is a borrow.

// Go: project/project.go:196 NewInferredProject
// PORT: Go `*core.CompilerOptions` (nil-able) is `Option<Rc<CompilerOptions>>`.
// PORT: Go `[]*core.ProjectReference` is `Option<Vec<ProjectReference>>`
// (the `ParsedOptions` field type; nil is `None`).

// Go: project/project.go:234 newSyntheticProject (ts#64204)
// PORT: Go `*core.CompilerOptions` can be nil; the Rust command line needs a
// value, so a nil one is the zero options.

// Go: project/project.go:258 newInferredProjectCommandLine (tsgo#4712)

// Go: project/project.go:270 NewProject
// ts#64319: takes the project ID; a configured project sets its config file
// name and path after (NewConfiguredProject).


// Go: project/project.go:392 Project.Clone (dirty.Cloneable)

// Go: project/project.go:89 `var _ ls.Project = (*Project)(nil)`

// PORT: projects are shared as `Rc<RefCell<Project>>`; this impl lets them
// coerce to `Rc<dyn ls::Project>` (Go passes the `*Project`).

// Go: project/project.go:494 CreateProgramResult
// PORT: `checker_pool` is the pool that the CreateCheckerPool closure made
// for `program` (Go `result.Program.GetCheckerPool().(*checkerPool)`).
