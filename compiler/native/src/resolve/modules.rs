// Module tree and module file loading (docs/ARCH.md section 1.2).
//
// A crate starts at its root file. `mod foo;` searches one directory: the
// current file's own directory when the file is a crate root, `mod.rs`,
// `lib.rs`, or `main.rs`, or when the file was itself loaded by a
// `#[path]` attribute (such a file owns its directory like `mod.rs`);
// for any other file `a.rs`, the directory `a/` next to it. Inline
// modules add one path segment each on top of the search directory of
// the file that holds them. In the search directory, `foo.rs` and
// `foo/mod.rs` are the two candidates: both is L0502 (there is no
// preference order), neither is L0503. `#[path = "..."]` overrides the
// file; it is relative to the directory of the file that holds the
// `mod` item, plus the inline module path segments when the item sits in
// inline modules, as in Rust. The caller passes this as `owns_dir`:
// true for a crate root file and for a `#[path]`-loaded file.
//
// A file belongs to exactly one (crate, module) pair per load, but the
// same file can be a module in two crates (shared part files): file text
// is cached per canonical path in the session, while AST, defs, and
// checks are per (crate, module) and never share a DefId.
use crate::diag::{Diag, Span};
use std::path::{Path, PathBuf};

/// True when `file`'s submodules live in the file's own directory: the
/// caller says the file owns its directory (a crate root or a
/// `#[path]`-loaded file), or the file is named `mod.rs`, `lib.rs`, or
/// `main.rs`.
pub fn is_dir_owner(file: &Path, owns_dir: bool) -> bool {
    if owns_dir {
        return true;
    }
    matches!(file.file_name().and_then(|n| n.to_str()), Some("mod.rs") | Some("lib.rs") | Some("main.rs"))
}

/// The directory `mod foo;` searches, for a declaration in `file` inside
/// the inline modules `inline_path` (segment names, outermost first).
pub fn search_dir(file: &Path, owns_dir: bool, inline_path: &[String]) -> PathBuf {
    let parent = file.parent().unwrap_or(Path::new(""));
    let mut dir = if is_dir_owner(file, owns_dir) {
        parent.to_path_buf()
    } else {
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        parent.join(stem)
    };
    for seg in inline_path {
        dir.push(seg);
    }
    dir
}

/// The base directory a `#[path]` value is relative to: the directory of
/// the file that holds the `mod` item, plus the inline module path. For a
/// `mod` item in a file module (no inline container) this is the file's
/// own directory even when the file is not a dir owner; that is Rust's
/// rule for `#[path]` and differs from the default search directory.
pub fn path_attr_base(file: &Path, owns_dir: bool, inline_path: &[String]) -> PathBuf {
    let dir = file.parent().unwrap_or(Path::new("")).to_path_buf();
    // A `#[path]` inside inline modules is relative to the *module*
    // directory: the search directory the inline module itself would
    // use, which adds the file stem only for a file that does not own
    // its directory (a crate root named anything, such as
    // `goport_util_root.rs`, searches its own directory).
    if !inline_path.is_empty() {
        return search_dir(file, owns_dir, inline_path);
    }
    dir
}

/// Resolves the file for one `mod name;` declaration. `path_attr` is the
/// `#[path]` value when written. Errors carry the declaration's span:
/// L0502 when both candidates exist, L0503 when the file is missing (or
/// the `#[path]` file is missing).
pub fn module_file(
    file: &Path,
    owns_dir: bool,
    inline_path: &[String],
    name: &str,
    path_attr: Option<&str>,
    span: Span,
) -> Result<PathBuf, Diag> {
    if let Some(p) = path_attr {
        let full = path_attr_base(file, owns_dir, inline_path).join(p);
        if full.is_file() {
            return Ok(full);
        }
        return Err(Diag::new(
            "L0503",
            span,
            format!("file not found for module `{name}`: `#[path]` names {}", full.display()),
            "Create the file, or fix the `#[path]` value.",
        ));
    }
    let (flat, nested) = module_candidates(file, owns_dir, inline_path, name);
    match (flat.is_file(), nested.is_file()) {
        (true, true) => Err(Diag::new(
            "L0502",
            span,
            format!("ambiguous module `{name}`: both {} and {} exist", flat.display(), nested.display()),
            "Delete one of the two files. Rust has no preference order.",
        )),
        (true, false) => Ok(flat),
        (false, true) => Ok(nested),
        (false, false) => Err(Diag::new(
            "L0503",
            span,
            format!("file not found for module `{name}`: tried {} and {}", flat.display(), nested.display()),
            "Create one of the two files, or add `#[path = \"...\"]` to the `mod` item.",
        )),
    }
}

/// The two files a `mod name;` without `#[path]` may name: `name.rs` and
/// `name/mod.rs` in the search directory.
pub fn module_candidates(file: &Path, owns_dir: bool, inline_path: &[String], name: &str) -> (PathBuf, PathBuf) {
    let dir = search_dir(file, owns_dir, inline_path);
    (dir.join(format!("{name}.rs")), dir.join(name).join("mod.rs"))
}

/// The canonical path string a file is keyed by in the session file
/// index and in disk keys (never the session-local FileId, section 1.2).
pub fn canonical_string(path: &Path) -> String {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lust-modules-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn search_dir_rules() {
        let src = Path::new("/x/src");
        // Crate root, mod.rs, lib.rs, main.rs: same directory.
        assert_eq!(search_dir(&src.join("main.rs"), true, &[]), src);
        assert_eq!(search_dir(&src.join("anything.rs"), true, &[]), src, "a crate root file of any name");
        assert_eq!(search_dir(&src.join("a/mod.rs"), false, &[]), src.join("a"));
        // Any other file a.rs: the a/ directory (the program.rs rule).
        assert_eq!(search_dir(&src.join("program.rs"), false, &[]), src.join("program"));
        // Inline modules add a segment each.
        assert_eq!(search_dir(&src.join("lib.rs"), true, &["inner".into()]), src.join("inner"));
        assert_eq!(search_dir(&src.join("a.rs"), false, &["inner".into()]), src.join("a/inner"));
        // A file loaded by #[path] owns its directory like mod.rs: its
        // children live next to it, not in a stem directory.
        assert_eq!(search_dir(&src.join("sub/m.rs"), true, &[]), src.join("sub"));
        assert_eq!(search_dir(&src.join("sub/m.rs"), true, &["inner".into()]), src.join("sub/inner"));
    }

    #[test]
    fn module_file_candidates() {
        let dir = tmp("candidates");
        let root = dir.join("main.rs");
        fs::write(dir.join("flat.rs"), "pub fn f() {}\n").unwrap();
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join("nested/mod.rs"), "pub fn f() {}\n").unwrap();
        fs::write(dir.join("both.rs"), "").unwrap();
        fs::create_dir_all(dir.join("both")).unwrap();
        fs::write(dir.join("both/mod.rs"), "").unwrap();
        let span = Span::default();
        assert_eq!(module_file(&root, true, &[], "flat", None, span).unwrap(), dir.join("flat.rs"));
        assert_eq!(module_file(&root, true, &[], "nested", None, span).unwrap(), dir.join("nested/mod.rs"));
        assert_eq!(module_file(&root, true, &[], "both", None, span).unwrap_err().code, "L0502");
        assert_eq!(module_file(&root, true, &[], "nope", None, span).unwrap_err().code, "L0503");
        // #[path] from the crate root file is relative to its directory.
        assert_eq!(module_file(&root, true, &[], "x", Some("flat.rs"), span).unwrap(), dir.join("flat.rs"));
        // #[path] in an inline module of a crate root file with a
        // non-dir-owner name: file directory plus inline segments,
        // with no file-stem segment.
        let named_root = dir.join("root_file.rs");
        fs::create_dir_all(dir.join("frontend")).unwrap();
        fs::write(dir.join("frontend/util_prelude.rs"), "").unwrap();
        assert_eq!(
            module_file(&named_root, true, &["frontend".into()], "prelude", Some("util_prelude.rs"), span).unwrap(),
            dir.join("frontend/util_prelude.rs")
        );
        assert_eq!(module_file(&root, true, &[], "x", Some("no.rs"), span).unwrap_err().code, "L0503");
        let _ = fs::remove_dir_all(&dir);
    }
}
