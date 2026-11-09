// K1 compatibility fixture: FileRef guard-backed borrow.
// panic-mode: abort
//
// Real shape: crates/ts_goport/src/ast/file_version.rs FileRef<T>
// (verbatim below): a Static &'static borrow, or a Pinned version
// guard (Rc<Arc<FileVersion>>) plus a key and a plain fn get that
// reads the data out of the pinned version. Deref returns a borrow
// whose lifetime is the guard, not the data definition. T may be
// unsized (str and slices below).
//
// Stand-in: FileVersion holds the two fields the gets read.

use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

/// One freeable file version: its text and its statement names.
struct FileVersion {
    text: String,
    names: Vec<String>,
}

/// ts-rust file_version.rs: a thread-local pin of one file version.
type VersionPin = Rc<Arc<FileVersion>>;

/// A borrow of per-file data. Verbatim from ts-rust file_version.rs.
pub enum FileRef<T: ?Sized + 'static> {
    Static(&'static T),
    Pinned {
        version: VersionPin,
        key: usize,
        get: fn(&FileVersion, usize) -> &T,
    },
}

impl<T: ?Sized + 'static> FileRef<T> {
    #[inline]
    #[must_use]
    pub fn as_static(&self) -> Option<&'static T> {
        match self {
            FileRef::Static(value) => Some(value),
            FileRef::Pinned { .. } => None,
        }
    }

    #[must_use]
    pub fn version(&self) -> Option<&VersionPin> {
        match self {
            FileRef::Static(_) => None,
            FileRef::Pinned { version, .. } => Some(version),
        }
    }
}

impl<T: ?Sized + 'static> Deref for FileRef<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            FileRef::Static(value) => value,
            FileRef::Pinned { version, key, get } => get(&**version, *key),
        }
    }
}

impl<T: ?Sized + 'static> Clone for FileRef<T> {
    fn clone(&self) -> Self {
        match self {
            FileRef::Static(value) => FileRef::Static(value),
            FileRef::Pinned { version, key, get } => FileRef::Pinned {
                version: Rc::clone(version),
                key: *key,
                get: *get,
            },
        }
    }
}

impl<T: 'static> Default for FileRef<[T]> {
    /// An empty static list.
    fn default() -> Self {
        FileRef::Static(&[])
    }
}

impl<T: ?Sized + std::fmt::Debug + 'static> std::fmt::Debug for FileRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

fn get_text(v: &FileVersion, _key: usize) -> &str {
    &v.text
}

fn get_name(v: &FileVersion, key: usize) -> &str {
    &v.names[key]
}

fn get_names(v: &FileVersion, _key: usize) -> &[String] {
    &v.names
}

static STATIC_TEXT: &str = "lib.d.ts text";

fn main() {
    // Static: a plain 'static borrow, no guard.
    let s: FileRef<str> = FileRef::Static(STATIC_TEXT);
    println!("static: {} (len {}) as_static={:?} pinned={}", &*s, s.len(), s.as_static(), s.version().is_some());

    // Pinned: the guard keeps the version alive past its owner.
    let pin: VersionPin = Rc::new(Arc::new(FileVersion {
        text: "const x = 1;".to_string(),
        names: vec!["x".to_string(), "y".to_string(), "z".to_string()],
    }));
    println!("arc strong before refs: {}", Arc::strong_count(&pin));
    let r_text: FileRef<str> = FileRef::Pinned {
        version: Rc::clone(&pin),
        key: 0,
        get: get_text,
    };
    let r_name: FileRef<str> = FileRef::Pinned {
        version: Rc::clone(&pin),
        key: 2,
        get: get_name,
    };
    let r_names: FileRef<[String]> = FileRef::Pinned {
        version: Rc::clone(&pin),
        key: 0,
        get: get_names,
    };
    let r_copy = r_name.clone();
    // The owner drops its pin; the guards keep the Arc alive.
    drop(pin);
    println!("pinned text: {:?}", &*r_text);
    println!("pinned name[2]: {:?} clone: {:?}", &*r_name, &*r_copy);
    println!("pinned names: {:?} count {}", &*r_names, r_names.len());
    println!("pinned as_static: {:?}", r_text.as_static());
    // The pin is Rc<Arc<..>>: guards clone the Rc, the Arc count
    // stays 1 while any guard lives.
    println!(
        "pin rc count with 4 guards: {} (arc {})",
        r_text.version().map_or(0, |v| Rc::strong_count(v)),
        r_text.version().map_or(0, |v| Arc::strong_count(v))
    );
    drop(r_names);
    drop(r_copy);
    drop(r_name);
    println!(
        "pin rc count with 1 guard: {} (arc {})",
        r_text.version().map_or(0, |v| Rc::strong_count(v)),
        r_text.version().map_or(0, |v| Arc::strong_count(v))
    );

    // Default slice ref (empty static), as ts-rust uses for no data.
    let empty: FileRef<[String]> = FileRef::default();
    println!("default empty: {} items", empty.len());
}
