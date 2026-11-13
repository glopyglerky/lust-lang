//! Port of modulespecifiers/util.go lines 398 to 502
//! (`ProcessEntrypointEnding`). Only `ls/autoimport` uses it.

use crate::prelude::*;

use crate::frontend::module::{Ending, ResolvedEntrypoint};

use super::deps;
use super::tspath;
use super::types::*;
use super::util::get_js_extension_for_declaration_file_extension;

// Go: modulespecifiers/util.go:388 ProcessEntrypointEnding
// ProcessEntrypointEnding processes a pre-computed module specifier from a package.json exports
// entrypoint according to the entrypoint's Ending type and the user's preferred endings.
// PORT: Go `module.TryGetJSExtensionForFile` is the package copy
// `deps::try_get_js_extension_for_file`. Go `strings.TrimSuffix` is
// `strip_suffix` with the unchanged string as the fallback.
