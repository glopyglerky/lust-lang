use crate::frontend::prelude::*;
use std::sync::LazyLock;

// Go: tsoptions/commandlineoption.go:9 CommandLineOptionKind
// PORT: Go `CommandLineOptionKind` is a string type. It stays a string
// newtype. Go `CommandLineOptionTypeX` constants are the associated
// constants `CommandLineOptionKind::X`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CommandLineOptionKind(pub &'static str);

impl CommandLineOptionKind {
    // Go: tsoptions/commandlineoption.go:12 CommandLineOptionTypeString
    pub const STRING: Self = Self("string");
    // Go: tsoptions/commandlineoption.go:13 CommandLineOptionTypeNumber
    pub const NUMBER: Self = Self("number");
    // Go: tsoptions/commandlineoption.go:14 CommandLineOptionTypeBoolean
    pub const BOOLEAN: Self = Self("boolean");
    // Go: tsoptions/commandlineoption.go:15 CommandLineOptionTypeObject
    pub const OBJECT: Self = Self("object");
    // Go: tsoptions/commandlineoption.go:16 CommandLineOptionTypeList
    pub const LIST: Self = Self("list");
    // Go: tsoptions/commandlineoption.go:17 CommandLineOptionTypeListOrElement
    pub const LIST_OR_ELEMENT: Self = Self("listOrElement");
    // Go: tsoptions/commandlineoption.go:18 CommandLineOptionTypeEnum
    pub const ENUM: Self = Self("enum"); // map
}



// Go: tsoptions/commandlineoption.go:21 CommandLineOption
// PORT: Go `string` fields of the static declarations are `&'static str`.
// Go `*diagnostics.Message` fields are `Option<&'static Message>`. Go
// unexported fields are plain `pub` fields. Declarations are leaked
// `&'static CommandLineOption` values, so Go pointer identity is
// `std::ptr::eq`.

// Go: tsoptions/commandlineoption.go:71 extraValidation
// PORT: Go string type kept as a string newtype.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ExtraValidation(pub &'static str);

impl ExtraValidation {
    // Go: tsoptions/commandlineoption.go:74 extraValidationNone
    pub const NONE: Self = Self("");
    // Go: tsoptions/commandlineoption.go:75 extraValidationSpec
    pub const SPEC: Self = Self("spec");
    // Go: tsoptions/commandlineoption.go:76 extraValidationLocale
    pub const LOCALE: Self = Self("locale");
}



// CommandLineOption.Elements()
// Go: tsoptions/commandlineoption.go:105 commandLineOptionElements
// PORT: Go package-level vars are `LazyLock` statics.

// CommandLineOption.EnumMap()
// Go: tsoptions/commandlineoption.go:183 commandLineOptionEnumMap

// CommandLineOption.DeprecatedKeys()
// Go: tsoptions/commandlineoption.go:197 commandLineOptionDeprecated
pub static COMMAND_LINE_OPTION_DEPRECATED: LazyLock<FxHashMap<&'static str, FxHashSet<String>>> =
    LazyLock::new(|| {
        let set = |items: &[&str]| {
            items
                .iter()
                .map(|s| s.to_string())
                .collect::<FxHashSet<String>>()
        };
        let mut m = FxHashMap::default();
        m.insert("module", set(&["none", "amd", "system", "umd"]));
        m.insert("moduleResolution", set(&["node", "classic", "node10"]));
        m.insert("target", set(&["es5"]));
        m
    });
