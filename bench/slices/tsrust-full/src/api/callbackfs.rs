//! Port of internal/api/callbackfs.go.

use crate::api::prelude::*;

use crate::frontend::json::{
    JsonDecoder, JsonError, MarshalerTo, UnmarshalerFrom, 
};
use crate::frontend::json_ext::{
    AnyValue, write_object_end, write_object_start,
};
use crate::frontend::vfs::{Entries, FileInfo, FsError};
use crate::gostd::{GoError, errors};
use std::time::SystemTime;

// Go: callbackfs.go:20 callbackFS
// callbackFS wraps a base filesystem and delegates certain operations
// to the client via RPC callbacks. This allows the API client to provide
// a virtual filesystem (e.g., in-memory files for testing).
//
// The callbacks to enable are specified at construction time via the
// --callbacks CLI flag. The connection is set via SetConnection after
// the transport connection is established.

// Go: callbackfs.go:29
// Callback names that can be enabled
const CALLBACK_READ_FILE: &str = "readFile";
const CALLBACK_FILE_EXISTS: &str = "fileExists";
const CALLBACK_DIRECTORY_EXISTS: &str = "directoryExists";
const CALLBACK_GET_ACCESSIBLE_ENTRIES: &str = "getAccessibleEntries";
const CALLBACK_REALPATH: &str = "realpath";
// tsgo#4699
const CALLBACK_WRITE_FILE: &str = "writeFile";
// ts#64158
const CALLBACK_REMOVE_FILE: &str = "removeFile";

// Go: callbackfs.go:40 isCallbackName
fn is_callback_name(name: &str) -> bool {
    matches!(
        name,
        CALLBACK_READ_FILE
            | CALLBACK_FILE_EXISTS
            | CALLBACK_DIRECTORY_EXISTS
            | CALLBACK_GET_ACCESSIBLE_ENTRIES
            | CALLBACK_REALPATH
            | CALLBACK_WRITE_FILE
            | CALLBACK_REMOVE_FILE
    )
}

// Go: callbackfs.go:58 newCallbackFS
// newCallbackFS creates a new callbackFS wrapping the given base filesystem.
// The callbacks slice specifies which filesystem operations should be delegated
// to the client (e.g., "readFile", "fileExists").

// Go: the anonymous `struct { Content *string `json:"content"` }` in
// ReadFile. Its Go type string holds the tag.
#[derive(Default)]
struct ReadFileWrapper {
    content: Option<String>,
}


// Go: the anonymous `struct { Files []string; Directories []string }` (with
// JSON tags) in GetAccessibleEntries. Its Go type string holds the tags.
#[derive(Default)]
struct RawEntries {
    files: Vec<String>,
    directories: Vec<String>,
}


// Go: the anonymous `struct { Path string; Data string }` in WriteFile
// (tsgo#4699).
#[derive(Debug)]
struct WriteFilePayload {
    path: String,
    data: String,
}



// Go `panic(err)`. The runtime prints an error value by its text.
#[track_caller]
fn panic_error(err: &GoError) -> ! {
    crate::core::go_panic(err.error())
}

