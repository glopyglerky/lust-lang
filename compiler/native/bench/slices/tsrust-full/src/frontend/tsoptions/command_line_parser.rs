use crate::frontend::prelude::*;

// This file ports tsoptions/commandlineparser.go.
// PORT: Go `any` values are `CompilerOptionsValue`. Go
// `*collections.OrderedMap[string, any]` is `IndexMap<String,
// CompilerOptionsValue>`. Go nil `*ast.Node` and `*ast.SourceFile` are
// `Node::NIL`.


// Go: tsoptions/commandlineparser.go:32 commandLineParser
// PORT: the Go `fs vfs.FS` field is not stored. errors.rs has `impl
// CommandLineParser` with no lifetime, so the parser cannot hold the
// borrowed `&dyn Fs` that `ParseConfigHost::fs` returns. The file system is
// passed to `parse_strings` and `parse_response_file` instead. The value is
// the same for the whole parse, as in Go.

// Go: tsoptions/commandlineparser.go:43 ParseCommandLine
// PORT: the plan names the entry point `parse_command_line(args, fs)`; Go
// takes a `ParseConfigHost`, so this does too.
// PORT: Go also converts the options with `watchOptionsParser` and stores the
// result in `ParsedConfig.WatchOptions`. Watch mode is out of scope, the
// crate has no `WatchOptions` type and U15 did not port `watchOptionsParser`,
// so that step is left out. It does not change the compiler options, the
// file names, the errors or the raw options.

// Go: tsoptions/commandlineparser.go:64 ParseBuildCommandLine
// PORT: ported with the other build mode types in
// `execute/build/command_line.rs` (`parse_build_command_line`).

// Go: tsoptions/commandlineparser.go:115 parseCommandLineWorker
// PORT: `fs` is passed down instead of stored (see `CommandLineParser`).
// Go nil `vfs.FS` is `None`.


// Go: tsoptions/commandlineparser.go:164 getInputOptionName


// Go: tsoptions/commandlineparser.go:224 tryReadFile


// Go: tsoptions/commandlineparser.go:347 ParseListTypeOption
// PORT: Go returns `[]any`: a `List`, or a `NilList` when `core.MapFiltered`
// keeps no element (for example `--types ,`), so the option stays unset.


// Go: tsoptions/commandlineparser.go:392 convertJsonOptionOfEnumType
