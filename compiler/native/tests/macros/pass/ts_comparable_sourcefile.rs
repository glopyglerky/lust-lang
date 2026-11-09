// macro: go_comparable_by_value, source_file_field
// source: crates/ts_goport/src/frontend/core_ls_ext.rs:19
// source: crates/ts_goport/src/api/encoder/encoder.rs:455
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
pub trait GoComparable { fn go_eq(&self, other: &Self) -> bool; }

macro_rules! go_comparable_by_value {
    ($($t:ty),* $(,)?) => {
        $(
            impl GoComparable for $t {
                fn go_eq(&self, other: &Self) -> bool {
                    self == other
                }
            }
        )*
    };
}

go_comparable_by_value!(u32, bool, char);

pub struct Info { pub name: String, pub version: u32, }
pub struct File { pub name: String, pub version: u32, }
pub enum SourceFileFields { Published(Info), Parsed(File), }

macro_rules! source_file_field {
    ($fields:expr, $field:ident) => {
        match $fields {
            SourceFileFields::Published(info) => &info.$field,
            SourceFileFields::Parsed(file) => &file.$field,
        }
    };
}

fn main() {
    println!("eq={} ne={}", 7u32.go_eq(&7), true.go_eq(&false));
    let a = SourceFileFields::Published(Info { name: "a.ts".into(), version: 3 });
    let b = SourceFileFields::Parsed(File { name: "b.ts".into(), version: 5 });
    let (ra, rb) = (&a, &b);
    println!("{}:{} {}:{}", source_file_field!(ra, name), source_file_field!(ra, version),
        source_file_field!(rb, name), source_file_field!(rb, version));
}
