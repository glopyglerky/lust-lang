use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/export.go` and `ls/autoimport/export_stringer_generated.go`.

use crate::flags_macros::go_enum;
use crate::frontend::tspath;
use crate::ls::lsutil;

// Go: ls/autoimport/export.go:17 ModuleID
// ModuleID uniquely identifies a module across multiple declarations.
// If the export is from an ambient module declaration, this is the module name.
// If the export is from a module augmentation, this is the Path() of the resolved module file.
// Otherwise this is the Path() of the exporting source file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleID(pub String);

// Go: ls/autoimport/export.go:19 ExportID
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ExportID {
    pub module_id: ModuleID,
    pub export_name: String,
}

// Go: ls/autoimport/export.go:24 ExportSyntax
go_enum!(ExportSyntax, i32 {
    NONE = 0; // ExportSyntaxNone
    // export const x = {}
    MODIFIER = 1; // ExportSyntaxModifier
    // export { x }
    NAMED = 2; // ExportSyntaxNamed
    // export default function f() {}
    DEFAULT_MODIFIER = 3; // ExportSyntaxDefaultModifier
    // export default f
    DEFAULT_DECLARATION = 4; // ExportSyntaxDefaultDeclaration
    // export = x
    EQUALS = 5; // ExportSyntaxEquals
    // export as namespace x
    UMD = 6; // ExportSyntaxUMD
    // export * from "module"
    STAR = 7; // ExportSyntaxStar
    // module.exports = {}
    COMMON_JS_MODULE_EXPORTS = 8; // ExportSyntaxCommonJSModuleExports
    // exports.x = {}
    COMMON_JS_EXPORTS_PROPERTY = 9; // ExportSyntaxCommonJSExportsProperty
});

// Go: ls/autoimport/export.go:48 Export
// PORT: Go embeds `ExportID`; it is the nested `export_id` field, and `Deref`
// promotes its fields (`export.module_id`, `export.export_name`) as Go does.
// Go `*Export` values are shared (`Rc<Export>`) once they are complete.
#[derive(Clone, Debug, Default)]
pub struct Export {
    pub export_id: ExportID,
    pub module_file_name: String,
    pub syntax: ExportSyntax,
    pub flags: SymbolFlags,
    pub local_name: String,
    // through is the name of the module symbol's export that this export was found on,
    // either 'export=', InternalSymbolNameExportStar, or empty string.
    pub through: String,

    // Checker-set fields
    pub target: ExportID,
    pub is_type_only: bool,
    pub script_element_kind: lsutil::ScriptElementKind,
    pub script_element_kind_modifiers: lsutil::ScriptElementKindModifier,

    // The file where the export was found.
    pub path: tspath::Path,

    pub package_name: String,
}

impl std::ops::Deref for Export {
    type Target = ExportID;
    fn deref(&self) -> &ExportID {
        &self.export_id
    }
}

impl std::ops::DerefMut for Export {
    fn deref_mut(&mut self) -> &mut ExportID {
        &mut self.export_id
    }
}

impl Export {
    // Go: ls/autoimport/export.go:71 Name
    pub fn name(&self) -> String {
        if !self.local_name.is_empty() {
            return self.local_name.clone();
        }
        if self.export_id.export_name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS {
            return self.target.export_name.clone();
        }
        self.export_id.export_name.clone()
    }

    // Go: ls/autoimport/export.go:81 IsRenameable
    pub fn is_renameable(&self) -> bool {
        self.export_id.export_name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            || self.export_id.export_name == INTERNAL_SYMBOL_NAME_DEFAULT
    }

    // Go: ls/autoimport/export.go:85 AmbientModuleName

    // Go: ls/autoimport/export.go:92 IsUnresolvedAlias
    pub fn is_unresolved_alias(&self) -> bool {
        self.flags == SymbolFlags::ALIAS
    }
}

// Go: ls/autoimport/export.go:96 SymbolToExport
// PORT: Go `*Export` result; nil is `None`.

// Go: ls/autoimport/export.go:128 tryGetModuleExport

// Go: ls/autoimport/export.go:136 extractFirstExport

// ---------------------------------------------------------------------------
// export_stringer_generated.go
// ---------------------------------------------------------------------------

// Go: ls/autoimport/export_stringer_generated.go:23 _ExportSyntax_name
const EXPORT_SYNTAX_NAME: &str = "ExportSyntaxNoneExportSyntaxModifierExportSyntaxNamedExportSyntaxDefaultModifierExportSyntaxDefaultDeclarationExportSyntaxEqualsExportSyntaxUMDExportSyntaxStarExportSyntaxCommonJSModuleExportsExportSyntaxCommonJSExportsProperty";

// Go: ls/autoimport/export_stringer_generated.go:25 _ExportSyntax_index
const EXPORT_SYNTAX_INDEX: [u8; 11] = [0, 16, 36, 53, 80, 110, 128, 143, 159, 192, 227];

impl ExportSyntax {
    // Go: ls/autoimport/export_stringer_generated.go:27 String
    pub fn string(self) -> String {
        let idx = self.0 - 0;
        if self.0 < 0 || idx as usize >= EXPORT_SYNTAX_INDEX.len() - 1 {
            return format!("ExportSyntax({})", self.0);
        }
        let idx = idx as usize;
        EXPORT_SYNTAX_NAME[EXPORT_SYNTAX_INDEX[idx] as usize..EXPORT_SYNTAX_INDEX[idx + 1] as usize]
            .to_string()
    }
}

impl std::fmt::Display for ExportSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string())
    }
}
