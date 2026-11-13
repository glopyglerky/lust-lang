//! Port of modulespecifiers/preferences.go.

use crate::prelude::*;

use super::tspath;
use super::types::*;

// Go: modulespecifiers/preferences.go:14 shouldAllowImportingTsExtension
// Program errors validate that `noEmit` or `emitDeclarationOnly` is also set,
// so this function doesn't check them to avoid propagating errors.

// Go: modulespecifiers/preferences.go:18 usesExtensionsOnImports

// Go: modulespecifiers/preferences.go:28 inferPreference

// Go: modulespecifiers/preferences.go:69 getModuleSpecifierEndingPreference

// Go: modulespecifiers/preferences.go:114 getPreferredEnding

// Go: modulespecifiers/preferences.go:141 ModuleSpecifierPreferences
// PORT: Go stores a closure over the arguments of getModuleSpecifierPreferences.
// The Rust closure borrows them for `'a`.
pub struct ModuleSpecifierPreferences<'a> {
    pub relative_preference: RelativePreferenceKind,
    pub get_allowed_endings_in_preferred_order:
        Box<dyn Fn(ResolutionMode) -> Vec<ModuleSpecifierEnding> + 'a>,
    pub exclude_regexes: Vec<String>,
}

// Go: modulespecifiers/preferences.go:147 GetAllowedEndingsInPreferredOrder

// Go: modulespecifiers/preferences.go:213 getModuleSpecifierPreferences
