//! Port of Go `internal/outputpaths` (outputpaths.go, commonsourcedirectory.go).

use crate::frontend::prelude::*;

// Go: outputpaths/outputpaths.go:9 OutputPathsHost
pub trait OutputPathsHost {
    fn common_source_directory(&self) -> String;
    /// #4712: the file extensions of the configured content mappers.
    fn content_mapper_extensions(&self) -> Vec<String>;
    fn get_current_directory(&self) -> String;
    fn use_case_sensitive_file_names(&self) -> bool;
}

// Go: outputpaths/outputpaths.go:16 OutputPaths
#[derive(Clone, Debug, Default)]
pub struct OutputPaths {
    js_file_path: String,
    source_map_file_path: String,
    declaration_file_path: String,
    declaration_map_path: String,
}

impl OutputPaths {
    // Go: outputpaths/outputpaths.go:25 (*OutputPaths).DeclarationFilePath
    pub fn declaration_file_path(&self) -> &str {
        &self.declaration_file_path
    }

    // Go: outputpaths/outputpaths.go:30 (*OutputPaths).JsFilePath
    pub fn js_file_path(&self) -> &str {
        &self.js_file_path
    }

    // Go: outputpaths/outputpaths.go:34 (*OutputPaths).SourceMapFilePath
    pub fn source_map_file_path(&self) -> &str {
        &self.source_map_file_path
    }

    // Go: outputpaths/outputpaths.go:38 (*OutputPaths).DeclarationMapPath
    pub fn declaration_map_path(&self) -> &str {
        &self.declaration_map_path
    }
}

// Go: `*outputpaths.OutputPaths` implements `declarations.OutputPaths`
// (transformers/declarations/transform.go:28).

// Go: outputpaths/outputpaths.go:41 ForceEmitPaths
/// Output paths to compute even when the options turn that output off (#4699).
/// The API emit and the builder signature emit set them.
#[derive(Clone, Copy, Debug, Default)]
pub struct ForceEmitPaths {
    pub dts: bool,
    pub js: bool,
    pub declaration_map: bool,
}

// Go: outputpaths/outputpaths.go:47 GetOutputPathsFor


// Go: outputpaths/outputpaths.go:72 ForEachEmittedFile

// Go: outputpaths/outputpaths.go:81 GetOutputJSFileName

// Go: outputpaths/outputpaths.go:97 isContentMappedFileName (#4712)

// Go: outputpaths/outputpaths.go:101 GetOutputJSFileNameWorker

// Go: outputpaths/outputpaths.go:108 GetOutputDeclarationFileNameWorker

// Go: outputpaths/outputpaths.go:116 GetOutputExtension
pub fn get_output_extension(file_name: &str, jsx: JsxEmit) -> &'static str {
    if file_extension_is(file_name, EXTENSION_JSON) {
        EXTENSION_JSON
    } else if jsx == JsxEmit::PRESERVE
        && file_extension_is_one_of(file_name, &[EXTENSION_JSX, EXTENSION_TSX])
    {
        EXTENSION_JSX
    } else if file_extension_is_one_of(file_name, &[EXTENSION_MTS, EXTENSION_MJS]) {
        EXTENSION_MJS
    } else if file_extension_is_one_of(file_name, &[EXTENSION_CTS, EXTENSION_CJS]) {
        EXTENSION_CJS
    } else {
        EXTENSION_JS
    }
}

// Go: outputpaths/outputpaths.go:131 GetDeclarationEmitOutputFilePath

// Go: outputpaths/outputpaths.go:148 ChangeToDeclarationExtension (#4712)

// Go: outputpaths/outputpaths.go:161 GetSourceFilePathInNewDir
// tsgo#4900: the same as the worker.

// Go: outputpaths/outputpaths.go:165 getOutputPathWithoutChangingExtension

// Go: outputpaths/outputpaths.go:175 GetSourceFilePathInNewDirWorker
// tsgo#4900: `TrimFilePathPrefix` cuts the common source directory by runes,
// not by its byte length.

// Go: outputpaths/outputpaths.go:183 getOwnEmitOutputFilePath

// Go: outputpaths/outputpaths.go:200 GetSourceMapFilePath

// Go: outputpaths/outputpaths.go:207 GetBuildInfoFileName

// Go: outputpaths/commonsourcedirectory.go:8 computeCommonSourceDirectoryOfFilenames

// Go: outputpaths/commonsourcedirectory.go:51 GetComputedCommonSourceDirectory

// Go: outputpaths/commonsourcedirectory.go:59 GetCommonSourceDirectory
// PORT: Go `checkSourceFilesBelongToPath` is a nillable callback; its
// result is unused.
