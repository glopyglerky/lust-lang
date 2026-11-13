//! Port of Go `compiler/emitter.go`: the per-file emitter, the script
//! transformer pipeline and the source map path helpers.
//!
//! `sourceFileMayBeEmitted`, `getSourceFilesToEmit`, `isSourceFileNotJson`
//! and `getDeclarationDiagnostics` are ported in `program.rs`.

use crate::prelude::*;

use super::program_emit::{SourceMapEmitResult, };
use crate::declarations::{
    
    
};
use crate::frontend::outputpaths::OutputPaths;
use crate::frontend::tspath::{
    ComparePathsOptions, ensure_trailing_directory_separator, file_extension_is,
    
    normalize_slashes,
};
use crate::sourcemap::generator::{Generator, new_generator};
use crate::transformers::transformer::{
    TransformReferenceResolver, 
};

// Go: compiler/emitter.go:24 EmitOnly
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmitOnly {
    #[default]
    All,
    Js,
    Dts,
    // #4849: renamed from Go `EmitOnlyForcedDts`.
    BuilderSignature,
}

// Go: compiler/emitter.go:33 emitter
// PORT: Go `tr *tracing.Tracing` is the process session
// (`crate::tracing::get`), so it is not a field.





// Go: compiler/emitter.go:55 declarationTransformer (#4712)
// PORT: renamed, because `DeclarationTransformer` is the declarations
// transformer struct.




// Go: compiler/emitter.go:295 declarationMapSource

// Go: compiler/emitter.go:301 newDeclarationMapSource

// Go: compiler/emitter.go:310 declarationMapSource.FileName, Text, ECMALineMap

// Go: compiler/emitter.go:90 getModuleTransformer





/// The choices Go `getScriptTransformers` makes from the options and the
/// file before it builds the transformers.
struct ScriptTransformChoice {
    import_elision_enabled: bool,
    jsx_transform_enabled: bool,
    /// The emit resolver (the checker) is the reference resolver. Else the
    /// binder reference resolver is.
    emit_resolver_references: bool,
}

// Go: compiler/emitter.go:112 getScriptTransformers (its first lines)



// Go: compiler/emitter.go:112 getScriptTransformers


// Go: compiler/emitter.go:402 shouldEmitSourceMaps

// Go: compiler/emitter.go:407 getSourceRoot

// Go: stringutil/util.go:144 EncodeURI
// PORT: Go escapes the bytes of the Go string. `s` is a port form (see
// `scanner_util::GO_STRING_MARKER`), so this escapes its Go bytes.

// Go: stringutil/util.go:164 shouldEscapeForEncodeURI

// Go: stringutil/util.go:215 AddUTF8ByteOrderMark
// PORT: a Rust `String` is UTF-8, so only the UTF-8 mark can be present.
pub fn add_utf8_byte_order_mark(text: String) -> String {
    if text.starts_with('\u{FEFF}') {
        return text;
    }
    format!("\u{FEFF}{text}")
}

/// A step of a file's emit where `set_emit_test_panic` makes it panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitTestPanic {
    /// After the declaration transforms of the file, on the thread that runs
    /// them (its checker thread).
    DeclarationTransforms,
    /// At the start of the JS print of the file, on the thread that prints
    /// it (its checker thread, the emit pool or the twin of its checker).
    JsPrint,
}

/// The panic that `set_emit_test_panic` set: the step and the end of the
/// file name.
static EMIT_TEST_PANIC: std::sync::Mutex<Option<(EmitTestPanic, String)>> =
    std::sync::Mutex::new(None);

/// True while `EMIT_TEST_PANIC` holds a panic, so the emit reads only this
/// flag when no test set one.
static EMIT_TEST_PANIC_ON: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);


