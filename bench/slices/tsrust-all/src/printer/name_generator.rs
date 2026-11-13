//! Port of Go `printer/namegenerator.go`.

use crate::prelude::*;

use super::emit_context::{AutoGenerateId, };
use super::types::GeneratedIdentifierFlags;
use super::utilities::{
    ensure_leading_hash, format_generated_name, 
    remove_leading_hash,
};

/// Go `tempFlags`. Flags enum to track count of temp variables and a few dedicated names
pub(crate) type TempFlags = i32;

pub(crate) const TEMP_FLAGS_AUTO: TempFlags = 0x0000_0000; // No preferred name
pub(crate) const TEMP_FLAGS_COUNT_MASK: TempFlags = 0x0FFF_FFFF; // Temp variable counter
pub(crate) const TEMP_FLAGS_I: TempFlags = 0x1000_0000; // Use/preference flag for '_i'

/// Go `func(string, bool) bool` (Printer.isFileLevelUniqueNameInCurrentFile).
pub type IsFileLevelUniqueNameFn = Rc<dyn Fn(&str, bool) -> bool>;


/// Go `nameGenerationScope`.
#[derive(Default)]
struct NameGenerationScope {
    next: Option<Box<NameGenerationScope>>, // The next nameGenerationScope in the stack
    temp_flags: TempFlags,                  // TempFlags for the current name generation scope.
    formatted_name_temp_flags: FxHashMap<String, TempFlags>, // TempFlags for the current name generation scope.
    reserved_names: FxHashSet<String>, // Names reserved in nested name generation scopes.
                                       // generatedNames         collections.Set[string] // NOTE: generated names should be scoped after Strada port is complete.
}


// Go: printer/namegenerator.go:355 nextContainer

// Go: printer/namegenerator.go:363 isUniqueLocalName
// PORT: Go reads `local.Flags` from the symbol. Locals are binder symbols, so
// this reads them from the program's bound symbol arena.
