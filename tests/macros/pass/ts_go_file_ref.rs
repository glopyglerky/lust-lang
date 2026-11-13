// macro: go_file_ref
// source: crates/ts_goport/src/ast/file_version.rs:925
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::rc::Rc;

pub mod ast {
    use std::ops::Deref;
    use std::rc::Rc;

    #[derive(Debug)]
    pub struct GoFile { pub info: i32, pub items: Vec<i32> }
    pub struct FileVersion { file: GoFile }
    impl FileVersion {
        pub fn go_file(&self) -> &GoFile { &self.file }
    }
    pub type VersionPin = Rc<FileVersion>;
    pub enum FileRef<T: ?Sized + 'static> {
        Static(&'static T),
        Pinned { version: VersionPin, key: usize, get: fn(&FileVersion, usize) -> &T },
    }
    impl<T: ?Sized + 'static> Deref for FileRef<T> {
        type Target = T;
        fn deref(&self) -> &T {
            match self {
                FileRef::Static(v) => v,
                FileRef::Pinned { version, key, get } => get(&**version, *key),
            }
        }
    }
    impl<T: ?Sized + 'static> Clone for FileRef<T> {
        fn clone(&self) -> Self {
            match self {
                FileRef::Static(v) => FileRef::Static(v),
                FileRef::Pinned { version, key, get } => FileRef::Pinned { version: Rc::clone(version), key: *key, get: *get },
            }
        }
    }
    pub fn go_file(file: usize) -> FileRef<GoFile> {
        if file == 0 {
            static F: std::sync::OnceLock<GoFile> = std::sync::OnceLock::new();
            FileRef::Static(F.get_or_init(|| GoFile { info: 10, items: vec![5, 6, 7] }))
        } else {
            FileRef::Pinned {
                version: Rc::new(FileVersion { file: GoFile { info: 20, items: vec![8, 9] } }),
                key: 0,
                get: |v, _k| v.go_file(),
            }
        }
    }
    pub mod file_version {
        macro_rules! go_file_ref {
    ($file:expr, $key:expr, |$g:ident, $k:ident| $part:expr) => {{
        let key: usize = $key;
        match $crate::ast::go_file($file) {
            $crate::ast::FileRef::Static($g) => {
                let $k = key;
                $crate::ast::FileRef::Static(&$part)
            }
            $crate::ast::FileRef::Pinned { version, .. } => $crate::ast::FileRef::Pinned {
                version,
                key,
                get: |version, $k| {
                    let $g = version.go_file();
                    &$part
                },
            },
        }
    }};
}
        pub(crate) use go_file_ref;
    }
}

fn main() {
    let info = crate::ast::file_version::go_file_ref!(0usize, 0usize, |g, _k| g.info);
    println!("static info={}", *info);
    let item = crate::ast::file_version::go_file_ref!(0usize, 2usize, |g, k| g.items[k]);
    println!("static item={}", *item);
    let pinned = crate::ast::file_version::go_file_ref!(1usize, 1usize, |g, k| g.items[k]);
    println!("pinned item={}", *pinned);
}
