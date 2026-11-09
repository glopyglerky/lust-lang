//! Port of Go `checker/checker.go` lines 4794-5714 (index constraint checks,
//! property initialization, interface/enum/module declarations, imports and
//! exports).

use crate::prelude::*;

// PORT: Go `tspath.IsExternalModuleNameRelative`, `tspath.FileExtensionIsOneOf`
// and their helpers. tspath has no port in this crate, so these private
// copies follow the Go code exactly.

// Go: tspath/path.go:954 PathIsRelative
fn path_is_relative_p06(path: &str) -> bool {
    // True if path is ".", "..", or starts with "./", "../", ".\\", or "..\\".
    if path == "." || path == ".." {
        return true;
    }
    let b = path.as_bytes();
    if b.len() >= 2 && b[0] == b'.' && (b[1] == b'/' || b[1] == b'\\') {
        return true;
    }
    if b.len() >= 3 && b[0] == b'.' && b[1] == b'.' && (b[2] == b'/' || b[2] == b'\\') {
        return true;
    }
    false
}

// Go: tspath/path.go IsVolumeCharacter
fn is_volume_character_p06(ch: u8) -> bool {
    (b'a'..=b'z').contains(&ch) || (b'A'..=b'Z').contains(&ch)
}

// Go: tspath/path.go:152 getFileUrlVolumeSeparatorEnd
fn get_file_url_volume_separator_end_p06(url: &[u8], start: usize) -> i32 {
    if url.len() <= start {
        return -1;
    }
    let ch0 = url[start];
    if ch0 == b':' {
        return (start + 1) as i32;
    }
    if ch0 == b'%' && url.len() > start + 2 && url[start + 1] == b'3' {
        let ch2 = url[start + 2];
        if ch2 == b'a' || ch2 == b'A' {
            return (start + 3) as i32;
        }
    }
    -1
}

// Go: tspath/path.go:169 GetEncodedRootLength

// Go: tspath/path.go:981 IsExternalModuleNameRelative

// Go: tspath/extension.go:79 FileExtensionIsOneOf
fn file_extension_is_one_of_p06(path: &str, extensions: &[&str]) -> bool {
    extensions
        .iter()
        .any(|ext| path.len() > ext.len() && path.ends_with(ext))
}


// Go: checker/checker.go:5010 (struct) InheritanceInfo



// Go: checker/checker.go:5530 hasTypeJsonImportAttribute


// Go: checker/checker.go:5740 isContainedByNamespace


// Go: checker/checker.go:5846 getVerbatimModuleSyntaxErrorMessage

