//! Go `cmd/tsc/api.go`.

use crate::cmd::tsgo::prelude::*;

use crate::cmd::tsgo::main::{ErrorHandling, new_flag_set, };
use crate::contentmapper::{self, ProcessExitState};
use crate::execute::tsc::stdio;
use crate::frontend::bundled;
use crate::gostd::context;
use std::io::Write;
use std::sync::Arc;

// Go: cmd/tsc/api.go:17 apiFlags (tsgo#4712)
struct ApiFlags {
    cwd: String,
    pipe_path: String,
    callbacks: String,
    async_: bool,
    timing: bool,
    run_external_code: bool,
}

// Go: cmd/tsc/api.go:26 parseAPIFlags (tsgo#4712)
// PORT: Go `StringVar` and `BoolVar` write into `result` as the flags are
// parsed. The port's flag set has only `String` and `Bool`, so the values
// are read into the result after the parse. The flag order and texts are
// Go's.



// Go: cmd/tsc/api.go:41 runAPI
