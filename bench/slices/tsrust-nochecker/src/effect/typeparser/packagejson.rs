//! Package typeparser provides Effect type detection and parsing utilities.
// Go: internal/typeparser/packagejson.go

use crate::effect::typeparser::*;
use crate::modulespecifiers::host::ProgramHost;
use crate::prelude::*;
use std::sync::Arc;


// Go: packageJsonProgram is an interface with GetSourceFileMetaData and
// GetPackageJsonInfo. The port's program has both: `get_source_file_meta_data`
// and `ProgramHost::get_package_json_info` (both read the current program of
// the thread, which is `tp.program`).

