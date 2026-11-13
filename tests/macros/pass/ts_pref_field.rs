// macro: pref_field
// source: crates/ts_goport/src/ls/lsutil/userpreferences.rs:890
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
#[derive(Debug)]
pub enum FieldValue { Int(i32), String(String) }
impl FieldValue {
    pub fn type_(&self) -> &'static str {
        match self { FieldValue::Int(_) => "int", FieldValue::String(_) => "string" }
    }
}
#[derive(Default)]
pub struct Editor { pub tab_size: i32, pub new_line: String }
#[derive(Default)]
pub struct Prefs { pub editor: Editor }
pub struct StructField {
    pub name: &'static str,
    pub raw_tag: &'static str,
    pub config_tag: &'static str,
    pub fallback_config_tag: &'static str,
    pub get: fn(&Prefs) -> FieldValue,
    pub set: fn(&mut Prefs, FieldValue),
}

macro_rules! pref_field {
    ($name:literal, $raw:literal, $config:literal, $variant:ident, $($path:ident).+) => {
        pref_field!($name, $raw, $config, fallback: "", $variant, $($path).+)
    };
    ($name:literal, $raw:literal, $config:literal, fallback: $fallback:literal, $variant:ident, $($path:ident).+) => {
        StructField {
            name: $name,
            raw_tag: $raw,
            config_tag: $config,
            fallback_config_tag: $fallback,
            get: |p| FieldValue::$variant(p.$($path).+.clone()),
            set: |p, v| match v {
                FieldValue::$variant(x) => p.$($path).+ = x,
                other => panic!(
                    "reflect.Set: value of type {:?} is not assignable to type {}",
                    other.type_(),
                    stringify!($variant)
                ),
            },
        }
    };
}

static FIELDS: &[StructField] = &[
    pref_field!("TabSize", "tabSize", "format.tabSize", Int, editor.tab_size),
    pref_field!("NewLine", "newLine", "format.newLine", fallback: "old.newLine", String, editor.new_line),
];

fn main() {
    let mut p = Prefs { editor: Editor { tab_size: 4, new_line: "\n".into() } };
    for f in FIELDS { println!("{} {} {:?} fb='{}'", f.name, f.config_tag, (f.get)(&p), f.fallback_config_tag); }
    (FIELDS[0].set)(&mut p, FieldValue::Int(8));
    println!("tab_size={}", p.editor.tab_size);
}
