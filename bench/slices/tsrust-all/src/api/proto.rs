//! Port of internal/api/proto.go.
//!
//! PORT: Go marshals and unmarshals the tagged structs by reflection (JSON
//! v2 default rules). Here each struct gets a hand-written `MarshalerTo`
//! and, when the server decodes it, an `UnmarshalerFrom`. `proto_json!`
//! writes both from a field list in Go declaration order with the Go tag
//! options (`plain`, `omitzero`, `omitempty`). Structs whose fields have no
//! JSON impls of their own (`TypeResponse`, `DiagnosticResponse`,
//! `ProjectResponse`, `ConfigFileResponse`, `SnapshotChanges`,
//! `ProjectFileChanges`) have their impls written out below the struct.
//!
//! Go `*T` fields are `Option<T>`; Go `[]*T` fields are `Vec<T>` (no Go
//! code stores a nil element). Go `any` in `TypeResponse.Value` only holds
//! JSON primitives, so it is `LspAny`.
//!
//! PORT: tsgo#4915 generates the TS API from this Go source
//! (`_tools/gen-proto`). Its `nonnil`, `deprecated` and `internal` struct
//! tags and the `@gen-proto-*` comments on the session handlers only steer
//! that generator. They do not change the JSON, so the port leaves them out.

use crate::api::prelude::*;

use crate::api::requestfilesystem;
use crate::execute::tsc::diagnostics as diagnosticwriter;
use crate::frontend::core_ext::ProjectReference;
use crate::frontend::json::{
    JsonDecoder, JsonError, JsonToken, MarshalerTo, UnmarshalerFrom, 
};
use crate::frontend::json_ext::{
    AnyValue, ErrorPos, IsZero, JsonValue, SemanticError, 
    
    write_object_end, write_object_start,
};
use crate::frontend::tspath;
use crate::gostd::{GoError, errors, strconv};
use crate::ls::lsconv;
use crate::lsp::lsproto;
use crate::project;
use std::borrow::Cow;
use std::sync::LazyLock;

/// Go JSON v2 default struct arshalers for the tagged proto structs.
///
/// `marshal` writes the members in Go declaration order. Each field names
/// its Go tag option: `plain` (always written), `omitzero` (skipped when
/// `IsZero`), `omitempty` (skipped when the value marshals as `null`, `""`,
/// `{}` or `[]`). `both` adds the v2 default unmarshal: names match
/// exactly, unknown names are skipped and `null` sets the zero value.
macro_rules! proto_json {
    (marshal $ty:ident { $($field:ident : $name:literal $mode:ident),* $(,)? }) => {
        impl MarshalerTo for $ty {
            fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
                write_object_start(enc);
                let mut first = true;
                $( proto_json!(@field $mode, enc, first, $name, &self.$field); )*
                write_object_end(enc);
                Ok(())
            }
        }
    };
    (both $ty:ident { $($field:ident : $name:literal $mode:ident),* $(,)? }) => {
        proto_json!(marshal $ty { $($field : $name $mode),* });

        impl UnmarshalerFrom for $ty {
            fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> {
                let is_object = unmarshal_struct_fields(
                    dec,
                    concat!("api.", stringify!($ty)),
                    |name, dec| {
                        match name {
                            $($name => json_unmarshal_decode(dec, &mut self.$field)?,)*
                            _ => return Ok(false),
                        }
                        Ok(true)
                    },
                )?;
                if !is_object {
                    *self = $ty::default();
                }
                Ok(())
            }
        }
    };
    (@field plain, $enc:ident, $first:ident, $name:literal, $value:expr) => {
        marshal_field($enc, &mut $first, $name, $value)?;
    };
    (@field omitzero, $enc:ident, $first:ident, $name:literal, $value:expr) => {
        marshal_field_omitzero($enc, &mut $first, $name, $value)?;
    };
    (@field omitempty, $enc:ident, $first:ident, $name:literal, $value:expr) => {
        marshal_field_omitempty($enc, &mut $first, $name, $value)?;
    };
}

// Go: proto.go:20

// Go: proto.go:36 Method
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Method(pub Cow<'static, str>);

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// PORT: `Method` is a Go string type; `BatchRequest.Method` and
// `BatchResponse.Method` (ts#63937) use the v2 string arshaler.


impl IsZero for Method {
    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
}

// Go: proto.go:27
// PORT: Go named integer and string types are newtypes. Their JSON form,
// zero test and `%v` text are those of the underlying Go type.
// ts#64319: Go `project::ID` is gone; the api uses `project::ID` (JSON below).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SnapshotID(pub u64);
// ts#64299
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleResolverID(pub u64);
// ts#64434
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceFileLeaseID(pub u64);
// ts#64158
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BuildOrchestratorID(pub u64);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolID(pub u64);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeID(pub u32);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignatureID(pub u64);
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeHandle(pub String);

// `uint` handles use the v2 uint arshaler, `string` handles the string
// arshaler. Unmarshal errors name the Go type (`api.SnapshotID`), not the
// underlying one.
macro_rules! handle_json {
    (@unmarshal uint, $self:ident, $dec:ident) => {{
        $self.0 = json_ext::unmarshal_uint_as($dec, &go_type_name::<Self>())?;
        Ok(())
    }};
    (@unmarshal string, $self:ident, $dec:ident) => {
        json_ext::unmarshal_string_as($dec, &mut $self.0, &go_type_name::<Self>())
    };
    ($kind:ident: $($name:ident),*) => {$(
        impl MarshalerTo for $name {
            fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
                self.0.marshal_json_to(enc)
            }
        }

        impl UnmarshalerFrom for $name {
            fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> {
                handle_json!(@unmarshal $kind, self, dec)
            }
        }

        impl IsZero for $name {
            fn is_zero(&self) -> bool {
                self.0.is_zero()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
    )*};
}


// Go: proto.go nextBuildOrchestratorId (ts#64158)
static NEXT_BUILD_ORCHESTRATOR_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

// Go: proto.go NewBuildOrchestratorID (ts#64158)
pub fn new_build_orchestrator_id() -> BuildOrchestratorID {
    BuildOrchestratorID(
        NEXT_BUILD_ORCHESTRATOR_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1,
    )
}

// PORT: Go `project.ID` and `project.SyntheticProjectID` (ts#64319) are Go
// string types: JSON uses the v2 string arshaler, except the Go
// `SyntheticProjectID.UnmarshalJSONFrom`, ported here. The project package
// has no JSON code for them, so their trait impls live with the protocol.


impl IsZero for project::ID {
    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
}


// Go: project/project.go SyntheticProjectID.UnmarshalJSONFrom (ts#64319)

impl IsZero for project::SyntheticProjectID {
    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
}

// Go: proto.go:55 SymbolHandle
// PORT: `symbols` is the arena that holds `symbol` (the rule for `ast`
// functions that take a symbol). Go dereferences a nil symbol and panics.

// Go: proto.go:59 TypeHandle
// PORT: Go reads `t.Id()`. A `TypeId` is the checker arena index, which
// equals the Go type id, so no checker is needed.
pub fn type_handle(t: TypeId) -> TypeID {
    if t.is_nil() {
        panic!("runtime error: invalid memory address or nil pointer dereference");
    }
    TypeID(t.0)
}

// Go: proto.go:63 SignatureHandle
// PORT: Go reads `sig.Id()`. A `SignatureId` is the checker arena index,
// which equals the Go signature id (`newSignature` numbers them from 1).
pub fn signature_handle(sig: SignatureId) -> SignatureID {
    if sig.is_nil() {
        panic!("runtime error: invalid memory address or nil pointer dereference");
    }
    SignatureID(u64::from(sig.0))
}

// ts#64319: Go `parseProjectHandle` is gone.

// Go: proto.go:56
impl Method {
    pub const RELEASE: Method = Method(Cow::Borrowed("release"));
    // ts#64434
    pub const RELEASE_SOURCE_FILE: Method = Method(Cow::Borrowed("releaseSourceFile"));

    // ts#63937
    pub const BATCH_REQUESTS: Method = Method(Cow::Borrowed("batchRequests"));
    // ts#64158
    pub const CREATE_BUILD_ORCHESTRATOR: Method = Method(Cow::Borrowed("createBuildOrchestrator"));
    pub const DISPOSE_BUILD_ORCHESTRATOR: Method =
        Method(Cow::Borrowed("disposeBuildOrchestrator"));
    pub const BUILD: Method = Method(Cow::Borrowed("build"));
    pub const BUILD_REFERENCES: Method = Method(Cow::Borrowed("buildReferences"));
    pub const CLEAN_BUILD: Method = Method(Cow::Borrowed("cleanBuild"));
    pub const CLEAN_REFERENCES: Method = Method(Cow::Borrowed("cleanReferences"));

    // tsgo#4915: MethodGetServerTiming and MethodResetServerTiming are gone;
    // the connection answers them (`ipc::timing`).
    pub const INITIALIZE: Method = Method(Cow::Borrowed("initialize"));
    // ts#64204: createSnapshot, updateSnapshot (new params) and
    // getCurrentLanguageServerSnapshot replace updateSnapshot and
    // updateTemporarySnapshot.
    pub const CREATE_SNAPSHOT: Method = Method(Cow::Borrowed("createSnapshot"));
    pub const UPDATE_SNAPSHOT: Method = Method(Cow::Borrowed("updateSnapshot"));
    pub const GET_CURRENT_LANGUAGE_SERVER_SNAPSHOT: Method =
        Method(Cow::Borrowed("getCurrentLanguageServerSnapshot"));
    // ts#64299
    pub const CREATE_MODULE_RESOLVER: Method = Method(Cow::Borrowed("createModuleResolver"));
    pub const RELEASE_MODULE_RESOLVER: Method = Method(Cow::Borrowed("releaseModuleResolver"));
    pub const RESOLVE_MODULE_NAME: Method = Method(Cow::Borrowed("resolveModuleName"));
    pub const PARSE_COMMAND_LINE: Method = Method(Cow::Borrowed("parseCommandLine"));
    pub const READ_CONFIG_FILE: Method = Method(Cow::Borrowed("readConfigFile"));
    pub const PARSE_JSON_CONFIG_FILE: Method = Method(Cow::Borrowed("parseJsonConfigFileContent"));
    pub const PARSE_CONFIG_FILE: Method = Method(Cow::Borrowed("parseConfigFile"));
    // ts#64216
    pub const CREATE_SOURCE_FILE: Method = Method(Cow::Borrowed("createSourceFile"));
    pub const CREATE_SOURCE_FILE_FROM_FILE: Method =
        Method(Cow::Borrowed("createSourceFileFromFile"));
    // tsgo#4849
    pub const TRANSPILE_MODULE: Method = Method(Cow::Borrowed("transpileModule"));
    pub const TRANSPILE_MODULE_FROM_FILE: Method = Method(Cow::Borrowed("transpileModuleFromFile"));
    pub const TRANSPILE_DECLARATION: Method = Method(Cow::Borrowed("transpileDeclaration"));
    pub const TRANSPILE_DECLARATION_FROM_FILE: Method =
        Method(Cow::Borrowed("transpileDeclarationFromFile"));
    pub const GET_DEFAULT_PROJECT_FOR_FILE: Method =
        Method(Cow::Borrowed("getDefaultProjectForFile"));
    pub const GET_SYMBOL_AT_POSITION: Method = Method(Cow::Borrowed("getSymbolAtPosition"));
    pub const GET_SYMBOLS_AT_POSITIONS: Method = Method(Cow::Borrowed("getSymbolsAtPositions"));
    pub const GET_SYMBOL_AT_LOCATION: Method = Method(Cow::Borrowed("getSymbolAtLocation"));
    pub const GET_SYMBOLS_AT_LOCATIONS: Method = Method(Cow::Borrowed("getSymbolsAtLocations"));
    pub const GET_SYMBOL_OF_SOURCE_FILE: Method = Method(Cow::Borrowed("getSymbolOfSourceFile"));
    pub const GET_SYMBOLS_OF_SOURCE_FILES: Method =
        Method(Cow::Borrowed("getSymbolsOfSourceFiles"));
    pub const GET_TYPE_OF_SYMBOL: Method = Method(Cow::Borrowed("getTypeOfSymbol"));
    pub const GET_TYPES_OF_SYMBOLS: Method = Method(Cow::Borrowed("getTypesOfSymbols"));
    pub const GET_DECLARED_TYPE_OF_SYMBOL: Method =
        Method(Cow::Borrowed("getDeclaredTypeOfSymbol"));
    // ts#63956
    pub const GET_NON_MISSING_TYPE_OF_SYMBOL: Method =
        Method(Cow::Borrowed("getNonMissingTypeOfSymbol"));
    pub const GET_SOURCE_FILE: Method = Method(Cow::Borrowed("getSourceFile"));
    pub const GET_SOURCE_FILE_NAMES: Method = Method(Cow::Borrowed("getSourceFileNames"));
    pub const GET_SOURCE_FILE_METADATA: Method = Method(Cow::Borrowed("getSourceFileMetadata"));
    // ts#64292
    pub const GET_MODE_FOR_USAGE_LOCATION: Method =
        Method(Cow::Borrowed("getModeForUsageLocation"));
    pub const GET_MODE_FOR_RESOLUTION_AT_INDEX: Method =
        Method(Cow::Borrowed("getModeForResolutionAtIndex"));
    // ts#64247
    pub const GET_RESOLVED_MODULE: Method = Method(Cow::Borrowed("getResolvedModule"));
    pub const GET_RESOLVED_MODULE_FROM_MODULE_SPECIFIER: Method =
        Method(Cow::Borrowed("getResolvedModuleFromModuleSpecifier"));
    pub const GET_RESOLVED_TYPE_REFERENCE_DIRECTIVE: Method =
        Method(Cow::Borrowed("getResolvedTypeReferenceDirective"));
    pub const GET_RESOLVED_TYPE_REFERENCE_DIRECTIVE_FROM_REFERENCE: Method = Method(Cow::Borrowed(
        "getResolvedTypeReferenceDirectiveFromTypeReferenceDirective",
    ));
    pub const GET_CONFIG_FILE_NAMES: Method = Method(Cow::Borrowed("getConfigFileNames"));
    pub const GET_CONFIG_SOURCE_FILE: Method = Method(Cow::Borrowed("getConfigSourceFile"));
    pub const RESOLVE_NAME: Method = Method(Cow::Borrowed("resolveName"));
    pub const GET_SYMBOLS_IN_SCOPE: Method = Method(Cow::Borrowed("getSymbolsInScope"));
    pub const GET_SIGNATURES_OF_TYPE: Method = Method(Cow::Borrowed("getSignaturesOfType"));
    pub const GET_RESOLVED_SIGNATURE: Method = Method(Cow::Borrowed("getResolvedSignature"));
    pub const GET_TYPE_AT_LOCATION: Method = Method(Cow::Borrowed("getTypeAtLocation"));
    pub const GET_TYPE_AT_LOCATIONS: Method = Method(Cow::Borrowed("getTypeAtLocations"));
    pub const GET_TYPE_AT_POSITION: Method = Method(Cow::Borrowed("getTypeAtPosition"));
    pub const GET_TYPES_AT_POSITIONS: Method = Method(Cow::Borrowed("getTypesAtPositions"));

    // Symbol sub-property methods
    pub const GET_PARENT_OF_SYMBOL: Method = Method(Cow::Borrowed("getParentOfSymbol"));
    pub const GET_MEMBERS_OF_SYMBOL: Method = Method(Cow::Borrowed("getMembersOfSymbol"));
    pub const GET_EXPORTS_OF_SYMBOL: Method = Method(Cow::Borrowed("getExportsOfSymbol"));
    pub const GET_EXPORT_SYMBOL_OF_SYMBOL: Method =
        Method(Cow::Borrowed("getExportSymbolOfSymbol"));

    // Type sub-property methods
    pub const GET_SYMBOL_OF_TYPE: Method = Method(Cow::Borrowed("getSymbolOfType"));
    pub const GET_TARGET_OF_TYPE: Method = Method(Cow::Borrowed("getTargetOfType"));
    pub const GET_FRESH_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getFreshTypeOfType"));
    pub const GET_REGULAR_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getRegularTypeOfType"));
    pub const GET_TYPES_OF_TYPE: Method = Method(Cow::Borrowed("getTypesOfType"));
    pub const GET_TYPE_PARAMETERS_OF_TYPE: Method =
        Method(Cow::Borrowed("getTypeParametersOfType"));
    pub const GET_OUTER_TYPE_PARAMETERS_OF_TYPE: Method =
        Method(Cow::Borrowed("getOuterTypeParametersOfType"));
    pub const GET_LOCAL_TYPE_PARAMETERS_OF_TYPE: Method =
        Method(Cow::Borrowed("getLocalTypeParametersOfType"));
    // ts#64264
    pub const GET_THIS_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getThisTypeOfType"));
    pub const GET_ALIAS_TYPE_ARGUMENTS_OF_TYPE: Method =
        Method(Cow::Borrowed("getAliasTypeArgumentsOfType"));
    pub const GET_ALIAS_SYMBOL_OF_TYPE: Method = Method(Cow::Borrowed("getAliasSymbolOfType"));
    pub const GET_OBJECT_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getObjectTypeOfType"));
    pub const GET_INDEX_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getIndexTypeOfType"));
    pub const GET_CHECK_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getCheckTypeOfType"));
    pub const GET_EXTENDS_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getExtendsTypeOfType"));
    pub const GET_BASE_TYPE_OF_TYPE: Method = Method(Cow::Borrowed("getBaseTypeOfType"));
    pub const GET_CONSTRAINT_OF_TYPE: Method = Method(Cow::Borrowed("getConstraintOfType"));
    // ts#64397
    pub const GET_TYPE_PARAMETER_OF_MAPPED_TYPE: Method =
        Method(Cow::Borrowed("getTypeParameterOfMappedType"));
    pub const GET_CONSTRAINT_TYPE_OF_MAPPED_TYPE: Method =
        Method(Cow::Borrowed("getConstraintTypeOfMappedType"));
    pub const GET_NAME_TYPE_OF_MAPPED_TYPE: Method =
        Method(Cow::Borrowed("getNameTypeOfMappedType"));
    pub const GET_TEMPLATE_TYPE_OF_MAPPED_TYPE: Method =
        Method(Cow::Borrowed("getTemplateTypeOfMappedType"));

    // Signature sub-property methods
    pub const GET_TYPE_PARAMETERS_OF_SIGNATURE: Method =
        Method(Cow::Borrowed("getTypeParametersOfSignature"));
    pub const GET_PARAMETERS_OF_SIGNATURE: Method =
        Method(Cow::Borrowed("getParametersOfSignature"));
    pub const GET_THIS_PARAMETER_OF_SIGNATURE: Method =
        Method(Cow::Borrowed("getThisParameterOfSignature"));
    pub const GET_TARGET_OF_SIGNATURE: Method = Method(Cow::Borrowed("getTargetOfSignature"));

    // Checker methods
    pub const GET_CONTEXTUAL_TYPE: Method = Method(Cow::Borrowed("getContextualType"));
    // ts#64264
    pub const GET_CONTEXTUAL_TYPE_FOR_ARGUMENT: Method =
        Method(Cow::Borrowed("getContextualTypeForArgument"));
    pub const GET_AWAITED_TYPE: Method = Method(Cow::Borrowed("getAwaitedType"));
    pub const GET_BASE_TYPE_OF_LITERAL_TYPE: Method =
        Method(Cow::Borrowed("getBaseTypeOfLiteralType"));
    pub const GET_NON_NULLABLE_TYPE: Method = Method(Cow::Borrowed("getNonNullableType"));
    pub const GET_TYPE_FROM_TYPE_NODE: Method = Method(Cow::Borrowed("getTypeFromTypeNode"));
    pub const GET_WIDENED_TYPE: Method = Method(Cow::Borrowed("getWidenedType"));
    pub const GET_PARAMETER_TYPE: Method = Method(Cow::Borrowed("getParameterType"));
    pub const GET_TYPE_PARAMETER_AT_POSITION: Method =
        Method(Cow::Borrowed("getTypeParameterAtPosition"));
    pub const IS_ARRAY_LIKE_TYPE: Method = Method(Cow::Borrowed("isArrayLikeType"));
    pub const IS_TYPE_ASSIGNABLE_TO: Method = Method(Cow::Borrowed("isTypeAssignableTo"));
    pub const GET_SHORTHAND_ASSIGNMENT_VALUE_SYMBOL: Method =
        Method(Cow::Borrowed("getShorthandAssignmentValueSymbol"));
    pub const GET_TYPE_OF_SYMBOL_AT_LOCATION: Method =
        Method(Cow::Borrowed("getTypeOfSymbolAtLocation"));
    pub const TYPE_TO_TYPE_NODE: Method = Method(Cow::Borrowed("typeToTypeNode"));
    pub const SIGNATURE_TO_SIGNATURE_DECLARATION: Method =
        Method(Cow::Borrowed("signatureToSignatureDeclaration"));
    pub const TYPE_TO_STRING: Method = Method(Cow::Borrowed("typeToString"));
    pub const IS_CONTEXT_SENSITIVE: Method = Method(Cow::Borrowed("isContextSensitive"));
    pub const GET_RETURN_TYPE_OF_SIGNATURE: Method =
        Method(Cow::Borrowed("getReturnTypeOfSignature"));
    pub const GET_REST_TYPE_OF_SIGNATURE: Method = Method(Cow::Borrowed("getRestTypeOfSignature"));
    pub const GET_TYPE_PREDICATE_OF_SIGNATURE: Method =
        Method(Cow::Borrowed("getTypePredicateOfSignature"));
    pub const GET_BASE_TYPES: Method = Method(Cow::Borrowed("getBaseTypes"));
    pub const GET_PROPERTIES_OF_TYPE: Method = Method(Cow::Borrowed("getPropertiesOfType"));
    pub const GET_APPARENT_PROPERTIES_OF_TYPE: Method =
        Method(Cow::Borrowed("getApparentPropertiesOfType"));
    pub const GET_APPARENT_TYPE: Method = Method(Cow::Borrowed("getApparentType"));
    // ts#63899
    pub const GET_REDUCED_TYPE: Method = Method(Cow::Borrowed("getReducedType"));
    pub const GET_PROPERTY_OF_TYPE: Method = Method(Cow::Borrowed("getPropertyOfType"));
    // ts#64264 (MethodGetIndexTypeOfTypeByKind is not ported: ts#64408
    // removes it)
    pub const GET_TYPE_OF_PROPERTY_OF_TYPE: Method =
        Method(Cow::Borrowed("getTypeOfPropertyOfType"));
    pub const GET_INDEX_INFO_OF_TYPE: Method = Method(Cow::Borrowed("getIndexInfoOfType"));
    pub const GET_INDEX_INFOS_OF_TYPE: Method = Method(Cow::Borrowed("getIndexInfosOfType"));
    pub const GET_CONSTRAINT_OF_TYPE_PARAMETER: Method =
        Method(Cow::Borrowed("getConstraintOfTypeParameter"));
    pub const GET_DEFAULT_FROM_TYPE_PARAMETER: Method =
        Method(Cow::Borrowed("getDefaultFromTypeParameter"));
    pub const GET_BASE_CONSTRAINT_OF_TYPE: Method =
        Method(Cow::Borrowed("getBaseConstraintOfType"));
    pub const GET_TYPE_ARGUMENTS: Method = Method(Cow::Borrowed("getTypeArguments"));
    // tsgo#3881
    pub const GET_IMPORT_ADDER_EDITS: Method = Method(Cow::Borrowed("getImportAdderEdits"));
    pub const GET_TRUE_TYPE_OF_CONDITIONAL_TYPE: Method =
        Method(Cow::Borrowed("getTrueTypeOfConditionalType"));
    pub const GET_FALSE_TYPE_OF_CONDITIONAL_TYPE: Method =
        Method(Cow::Borrowed("getFalseTypeOfConditionalType"));
    pub const GET_CONSTANT_VALUE: Method = Method(Cow::Borrowed("getConstantValue"));
    pub const GET_SIGNATURE_FROM_DECLARATION: Method =
        Method(Cow::Borrowed("getSignatureFromDeclaration"));
    pub const GET_EXPORT_SPECIFIER_LOCAL_TARGET: Method =
        Method(Cow::Borrowed("getExportSpecifierLocalTargetSymbol"));
    pub const GET_ALIASED_SYMBOL: Method = Method(Cow::Borrowed("getAliasedSymbol"));
    pub const GET_IMMEDIATE_ALIASED_SYMBOL: Method =
        Method(Cow::Borrowed("getImmediateAliasedSymbol"));
    // ts#63945
    pub const GET_TARGET_SYMBOL: Method = Method(Cow::Borrowed("getTargetSymbol"));
    // ts#64264
    pub const GET_EXPORT_SYMBOL_OF_SYMBOL_FOR_CHECKER: Method =
        Method(Cow::Borrowed("getExportSymbolOfSymbolForChecker"));
    pub const GET_FULLY_QUALIFIED_NAME: Method = Method(Cow::Borrowed("getFullyQualifiedName"));
    pub const GET_EXPORTS_OF_MODULE: Method = Method(Cow::Borrowed("getExportsOfModule"));
    pub const GET_MEMBER_IN_MODULE_EXPORTS: Method =
        Method(Cow::Borrowed("getMemberInModuleExports"));
    pub const GET_JS_DOC_TAGS: Method = Method(Cow::Borrowed("getJsDocTags"));
    pub const GET_DOCUMENTATION_COMMENT: Method = Method(Cow::Borrowed("getDocumentationComment"));
    pub const IS_ARRAY_TYPE: Method = Method(Cow::Borrowed("isArrayType"));
    // ts#64080: MethodIsTupleType is gone; TypeResponse.IsTupleType replaces it.
    // ts#63943
    pub const IS_READONLY_SYMBOL: Method = Method(Cow::Borrowed("isReadonlySymbol"));

    // Reference methods
    pub const GET_REFERENCES_TO_SYMBOL_IN_FILE: Method =
        Method(Cow::Borrowed("getReferencesToSymbolInFile"));
    pub const GET_REFERENCED_SYMBOLS_FOR_NODE: Method =
        Method(Cow::Borrowed("getReferencedSymbolsForNode"));
    pub const GET_SIGNATURE_USAGES: Method = Method(Cow::Borrowed("getSignatureUsages"));

    // Language service methods
    pub const GET_COMPLETIONS_AT_POSITION: Method =
        Method(Cow::Borrowed("getCompletionsAtPosition"));

    // Diagnostic methods
    pub const GET_SYNTACTIC_DIAGNOSTICS: Method = Method(Cow::Borrowed("getSyntacticDiagnostics"));
    pub const GET_BIND_DIAGNOSTICS: Method = Method(Cow::Borrowed("getBindDiagnostics"));
    pub const GET_SEMANTIC_DIAGNOSTICS: Method = Method(Cow::Borrowed("getSemanticDiagnostics"));
    pub const GET_SUGGESTION_DIAGNOSTICS: Method =
        Method(Cow::Borrowed("getSuggestionDiagnostics"));
    pub const GET_DECLARATION_DIAGNOSTICS: Method =
        Method(Cow::Borrowed("getDeclarationDiagnostics"));
    pub const GET_PROGRAM_DIAGNOSTICS: Method = Method(Cow::Borrowed("getProgramDiagnostics"));
    pub const GET_GLOBAL_DIAGNOSTICS: Method = Method(Cow::Borrowed("getGlobalDiagnostics"));
    pub const GET_CONFIG_FILE_PARSING_DIAGNOSTICS: Method =
        Method(Cow::Borrowed("getConfigFileParsingDiagnostics"));

    // Printer methods
    pub const PRINT_NODE: Method = Method(Cow::Borrowed("printNode"));
    pub const FORMAT_NODE_FOR_INSERTION: Method = Method(Cow::Borrowed("formatNodeForInsertion"));
    // tsgo#4699
    pub const EMIT: Method = Method(Cow::Borrowed("emit"));
    pub const EMIT_TO_STRING: Method = Method(Cow::Borrowed("emitToString"));
    pub const GET_JAVA_SCRIPT_EMIT: Method = Method(Cow::Borrowed("getJavaScriptEmit"));
    pub const GET_DECLARATION_EMIT: Method = Method(Cow::Borrowed("getDeclarationEmit"));

    // Intrinsic type getters
    pub const GET_ANY_TYPE: Method = Method(Cow::Borrowed("getAnyType"));
    pub const GET_STRING_TYPE: Method = Method(Cow::Borrowed("getStringType"));
    pub const GET_NUMBER_TYPE: Method = Method(Cow::Borrowed("getNumberType"));
    pub const GET_BOOLEAN_TYPE: Method = Method(Cow::Borrowed("getBooleanType"));
    pub const GET_VOID_TYPE: Method = Method(Cow::Borrowed("getVoidType"));
    pub const GET_UNDEFINED_TYPE: Method = Method(Cow::Borrowed("getUndefinedType"));
    pub const GET_NULL_TYPE: Method = Method(Cow::Borrowed("getNullType"));
    pub const GET_NEVER_TYPE: Method = Method(Cow::Borrowed("getNeverType"));
    pub const GET_UNKNOWN_TYPE: Method = Method(Cow::Borrowed("getUnknownType"));
    pub const GET_BIG_INT_TYPE: Method = Method(Cow::Borrowed("getBigIntType"));
    pub const GET_ES_SYMBOL_TYPE: Method = Method(Cow::Borrowed("getESSymbolType"));
    pub const GET_NON_PRIMITIVE_TYPE: Method = Method(Cow::Borrowed("getNonPrimitiveType"));

    // Well-known per-checker symbols
    pub const GET_WELL_KNOWN_SYMBOLS: Method = Method(Cow::Borrowed("getWellKnownSymbols"));

    // Well-known per-checker signatures
    pub const GET_WELL_KNOWN_SIGNATURES: Method = Method(Cow::Borrowed("getWellKnownSignatures"));

    // Profiling methods
    pub const START_CPU_PROFILE: Method = Method(Cow::Borrowed("startCPUProfile"));
    pub const STOP_CPU_PROFILE: Method = Method(Cow::Borrowed("stopCPUProfile"));
    pub const SAVE_HEAP_PROFILE: Method = Method(Cow::Borrowed("saveHeapProfile"));
}

// InitializeResponse is returned by the initialize method.
// Go: proto.go:265 InitializeResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InitializeResponse {
    // UseCaseSensitiveFileNames indicates whether the host file system is case-sensitive.
    pub use_case_sensitive_file_names: bool,
    // CurrentDirectory is the server's current working directory.
    pub current_directory: String,
}


// DocumentIdentifier identifies a document by either a file name (plain string) or a URI object.
// On the wire it is string | { uri: string }.
//
// @example
//
// Using a file name:
//
//	project.program.getSourceFile("/path/to/file.ts");
//
// Using a URI:
//
//	project.program.getSourceFile({ uri: "file:///path/to/file.ts" });
// Go: proto.go:284 DocumentIdentifier
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentIdentifier {
    pub file_name: String,
    pub uri: lsproto::DocumentUri,
}


// Go: proto.go:187 UnmarshalJSONFrom

// Go jsontext `Token.String`: the unquoted text of a string token, else the
// raw JSON text of the token.
fn token_string(tok: &JsonToken) -> String {
    match tok {
        JsonToken::Null => "null".to_string(),
        JsonToken::False => "false".to_string(),
        JsonToken::True => "true".to_string(),
        JsonToken::String(s) => s.clone(),
        JsonToken::Number(raw) => raw.clone(),
        JsonToken::BeginObject => "{".to_string(),
        JsonToken::EndObject => "}".to_string(),
        JsonToken::BeginArray => "[".to_string(),
        JsonToken::EndArray => "]".to_string(),
    }
}

// Go jsontext `Kind.String` (the `%v` text of a token kind).

impl DocumentIdentifier {
    // Go: proto.go:327 ToFileName

    // Go: proto.go:337 ToURI
    // ToURI returns the document URI for this identifier. An explicitly provided URI
    // is returned as-is; a file name is first normalized to an absolute path against
    // cwd before being converted to a URI.

    // Go: proto.go:344 ToAbsoluteFileName

    // Go: proto.go:351 String
    pub fn string(&self) -> String {
        if !self.uri.0.is_empty() {
            return self.uri.0.clone();
        }
        self.file_name.clone()
    }
}

// Go `%v` of a DocumentIdentifier calls its String method.
impl std::fmt::Display for DocumentIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string())
    }
}

// FileNotifications describes changes to files that have occurred on the host
// file system, used to notify the session to reload cached files and reevaluate
// tsconfig.json `include` globs. Either InvalidateAll is true (discard all caches)
// or Changed/Created/Deleted list individual documents.
// Go: proto.go FileNotifications (ts#64204; was APIFileChanges)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileNotifications {
    pub invalidate_all: bool,
    pub changed: Vec<DocumentIdentifier>,
    pub created: Vec<DocumentIdentifier>,
    pub deleted: Vec<DocumentIdentifier>,
}


// SnapshotRequestChangesParams describes project, file, and program changes to apply
// while creating or updating a snapshot.
// Go: proto.go SnapshotRequestChangesParams (ts#64204, ts#64319, ts#64374)
// PORT: Go `[]*CreateSnapshotProgramParams` can hold nil elements (a JSON
// `null`), which the session rejects, so they are `Vec<Option<..>>`. Go
// `CreatePrograms != nil` (an empty array included) is `Some`.



// Go: proto.go EnsurePrograms (ts#64204, ts#64319)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnsurePrograms {
    pub all: bool,
    pub projects: Vec<project::ID>,
}

// Go: proto.go EnsurePrograms.UnmarshalJSONFrom (ts#64204)

// PORT: Go has no marshaler for `EnsurePrograms`, so it marshals by the v2
// default struct rule: the untagged fields keep their Go names.

// CreateSnapshotParams are the parameters for creating a new independent snapshot.
// Go: proto.go CreateSnapshotParams (ts#64204, ts#64115)


// PORT: the server only decodes the snapshot params, but a decoded payload
// is a Go `any` (`AnyValue`), which marshals. The marshalers below follow
// the v2 default struct rule, so they match Go reflection.

// PORT: Go v2 default marshal of the `requestfilesystem` request structs
// (`CreateSnapshotParams.FileSystem`). They live here with the other
// params marshalers. Go map members come in random order; the port writes
// them in sorted key order.




/// Go v2 marshal of a `map[string]V` (a nil map writes `{}`), with the
/// members in sorted key order.
struct SortedMapJSON<'a, V>(&'a FxHashMap<String, V>);


// Go: proto.go CreateSnapshotProgramParams (ts#64204, ts#64324)

// PORT: the params are only decoded (`core.CompilerOptions` has no plain
// marshaler in the port), so only the decode is written.


// Go: proto.go ReconfigureSnapshotProgramParams (ts#64204, ts#64319, ts#64324)



// UpdateSnapshotParams are the parameters for deriving a snapshot from an existing one.
// Go: proto.go UpdateSnapshotParams (ts#64204)


// Go: proto.go GetCurrentLanguageServerSnapshotParams (ts#64204)


// LanguageServerSnapshotChanges describes API-driven changes to adopt into the
// language server's canonical state.
// Go: proto.go LanguageServerSnapshotChanges (ts#64204)



// PORT: Go decodes `core.ProjectReference` by reflection (the fields
// `path`, `originalPath` and `circular`); only the api decodes it.

// Go: proto.go CreateProgramOptions (ts#63950, ts#64324)
// PORT: Go `[]*core.ProjectReference` elements are never nil here.



// Go: proto.go ModuleResolutionFallback (ts#64299)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleResolutionFallback(pub Cow<'static, str>);

impl ModuleResolutionFallback {
    pub const RESOLVE: ModuleResolutionFallback =
        ModuleResolutionFallback(Cow::Borrowed("resolve"));
    pub const UNRESOLVED: ModuleResolutionFallback =
        ModuleResolutionFallback(Cow::Borrowed("unresolved"));
}



// Go: proto.go ResolutionMode (ts#64299): `type ResolutionMode core.ModuleKind`.
// PORT: named `ResolutionMode` as in Go; it is not the core alias
// (`crate::options::ResolutionMode`), which api code outside this file names
// `ModuleKind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ResolutionMode(pub i32);

impl MarshalerTo for ResolutionMode {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        self.0.marshal_json_to(enc)
    }
}


// Go: proto.go ModuleResolutionSpec (ts#64299)
// PORT: Go `[]*ModuleResolutionEntry` elements can be nil (JSON `null`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModuleResolutionSpec {
    pub fallback: ModuleResolutionFallback,
    pub entries: Vec<Option<ModuleResolutionEntry>>,
}



// Go: proto.go ModuleResolutionEntry (ts#64299)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModuleResolutionEntry {
    pub module_name: String,
    pub containing_directory: Option<DocumentIdentifier>,
    pub resolution_mode: Option<ResolutionMode>,
    pub result: Option<StaticModuleResolution>,
}



// Go: proto.go StaticModuleResolution (ts#64299)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StaticModuleResolution {
    pub resolved_file_name: Option<DocumentIdentifier>,
    pub original_path: Option<DocumentIdentifier>,
    pub package_id: Option<PackageId>,
}



// Go: proto.go CreateModuleResolverParams (ts#64299)



// Go: proto.go ReleaseModuleResolverParams (ts#64299)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReleaseModuleResolverParams {
    pub resolver: ModuleResolverID,
}


// Go: proto.go ResolveModuleNameParams (ts#64299)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolveModuleNameParams {
    pub snapshot: SnapshotID,
    pub in_progress_snapshot: u64,
    pub resolver: ModuleResolverID,
    pub module_name: String,
    pub containing_directory: DocumentIdentifier,
    pub resolution_mode: Option<ResolutionMode>,
}



// Go: proto.go ResolveModuleNameCallbackParams (ts#64299)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolveModuleNameCallbackParams {
    pub module_name: String,
    pub containing_directory: String,
    pub resolution_mode: Option<ResolutionMode>,
    pub snapshot: Option<SnapshotID>,
    pub in_progress_snapshot: Option<u64>,
}


// Go: proto.go ResolveModuleNameResult (ts#64299)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolveModuleNameResult {
    pub resolved_module: Option<ResolvedModule>,
    // Trace is provided when compilerOptions.traceResolution is true.
    pub trace: Vec<String>,
}


// ProjectFileChanges describes what source files changed within a single project.
// Go: proto.go:528 ProjectFileChanges
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectFileChanges {
    // ChangedFiles lists source file paths whose content differs.
    pub changed_files: Vec<tspath::Path>,
    // DeletedFiles lists source file paths removed from the project's program.
    pub deleted_files: Vec<tspath::Path>,
}


// SnapshotChanges describes what changed between a response base and a new
// snapshot. Changes are reported per-project so clients
// can track cache refs at the (snapshot, project) level.
// Go: proto.go SnapshotChanges (ts#64204, ts#64319)


// Go v2 map marshal of `map[project.ID]*ProjectFileChanges`: an object with
// the handles as names.
// PORT: Go map order is random; the port writes insertion order. Go values
// are never nil pointers, so the map holds values.
struct ProjectChangesJSON<'a>(&'a IndexMap<project::ID, ProjectFileChanges>);


// CreateSnapshotResponse is returned by createSnapshot.
// Go: proto.go CreateSnapshotResponse (ts#64204; was UpdateSnapshotResponse)


// Go: proto.go SnapshotOperationResponse (ts#64204, ts#64319)
// PORT: Go `*[]T` with `omitzero`: `None` is the nil pointer (omitted);
// `Some(vec![])` writes `[]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapshotOperationResponse {
    pub created_programs: Option<Vec<project::SyntheticProjectID>>,
    pub opened_files: Option<Vec<OpenedFileOperationResult>>,
}


// Go: proto.go OpenedFileOperationResult (ts#64204, ts#64319)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpenedFileOperationResult {
    pub project: project::ID,
}


/// Go `func([]byte) (any, error)` in `unmarshalers`.
pub type Unmarshaler = fn(&[u8]) -> Result<Option<Box<dyn AnyValue>>, GoError>;

// Go: proto.go:569 unmarshalers

// Go: proto.go:746 ParseConfigFileParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParseConfigFileParams {
    pub file: DocumentIdentifier,
}


// Go: proto.go:750 ParseCommandLineParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParseCommandLineParams {
    pub command_line: Vec<String>,
}


// Go: proto.go:754 ReadConfigFileParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReadConfigFileParams {
    pub file: DocumentIdentifier,
}


// Go: proto.go:758 ParseJsonConfigFileContentParams
// PORT: Go `JSON packagejson.JSONValue` is `LspAny`. A decoded request value
// must be `Send` (`AnyValue`), and `packagejson::JSONValue` holds `Rc`.
// Both decode a JSON value to the same tree: null, bool, float64, string,
// array, and an object that keeps its member order.


// Go: proto.go:764 jsonValueToAny
// PORT: Go `any` from the tsoptions JSON code is `CompilerOptionsValue`
// (nil, string, float64, bool, `[]any`, `*collections.OrderedMap`). The
// input is the `LspAny` that stands for `packagejson.JSONValue` (see
// `ParseJsonConfigFileContentParams`); Go's "not present" is `Null` there.

// Go: proto.go:789 TranspileOptions (tsgo#4849)
// PORT: Go `*core.CompilerOptions` is `Option<CompilerOptions>` (nil is
// `None`). Its JSON form is the Go struct default (`CompilerOptionsJSON` to
// write, the `CompilerOptions` `UnmarshalerFrom` below to read).



// Go: proto.go CreateSourceFileOptions (ts#64216)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreateSourceFileOptions {
    pub script_kind: ScriptKind,
}


// Go: proto.go CreateSourceFileParams (ts#64216)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreateSourceFileParams {
    pub file_name: String,
    pub source_text: String,
    pub options: CreateSourceFileOptions,
}


// Go: proto.go CreateSourceFileFromFileParams (ts#64216)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreateSourceFileFromFileParams {
    pub file_name: String,
    pub options: CreateSourceFileOptions,
}


// Go: proto.go:810 TranspileParams (tsgo#4849)


// Go: proto.go:815 TranspileFromFileParams (tsgo#4849)


// Go: proto.go:820 TranspileOutputResponse (tsgo#4849)


// Go: proto.go BatchRequestsParams (ts#63937)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatchRequestsParams {
    pub requests: Vec<BatchRequest>,
    // ts#64061
    pub continuation_token: String,
    pub max_response_bytes_per_page: i32,
}


// Go: proto.go BatchRequest (ts#63937)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatchRequest {
    pub method: Method,
    pub params: JsonValue,
}


// Go: proto.go BatchRequestsResponse (ts#63937, ts#64061)
// PORT: Go `encodedResponses []json.Value` is `Option`: `None` is nil.

// Go: proto.go BatchRequestsResponse.MarshalJSONTo (ts#64061)

// Go: proto.go BatchResponse (ts#63937)
// PORT: Go `Result any` is the handler result (`None` is a nil `any`).


// ReleaseParams are the parameters for the release method.
// Go: proto.go:889 ReleaseParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReleaseParams {
    pub snapshot: SnapshotID,
}


// Go: proto.go CreateBuildOrchestratorParams (ts#64158)
// PORT: Go embeds `*core.BuildOptions` and `*core.CompilerOptions` with
// JSON names; they decode as the members `buildOptions` and
// `compilerOptions`.



/// Go v2 marshal of `core.BuildOptions` (by reflection: the tags `dry`,
/// `force`, `verbose`, `builders`, `stopBuildOnErrors` and `clean`, each
/// `omitzero`). A `Tristate` writes its legacy `MarshalJSON` text.
struct BuildOptionsJSON<'a>(&'a crate::execute::build::BuildOptions);


// PORT: the Go v2 struct decode of `core.BuildOptions` (the tags
// `dry`, `force`, `verbose`, `builders`, `stopBuildOnErrors`, `clean`).
// `core.Tristate` decodes through its legacy `UnmarshalJSON` (raw value).

// Go: proto.go CreateBuildOrchestratorResponse (ts#64158)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreateBuildOrchestratorResponse {
    pub build_orchestrator_id: BuildOrchestratorID,
}


// Go: proto.go DisposeBuildOrchestratorParams (ts#64158)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisposeBuildOrchestratorParams {
    pub build_orchestrator_id: BuildOrchestratorID,
}


// Go: proto.go BuildParams (ts#64158)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuildParams {
    pub build_orchestrator_id: BuildOrchestratorID,
    pub project: String,
}




// Go: proto.go BuildResponse (ts#64158)
// PORT: Go `tsc.ExitStatus` is a Go int type; it writes as a number.


// Go: proto.go CleanBuildParams (ts#64158)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CleanBuildParams {
    pub build_orchestrator_id: BuildOrchestratorID,
    pub project: String,
}


// Go: proto.go CleanBuildResponse (ts#64158)


// PORT: Go `BuildOrchestrator` (a struct of three func fields, ts#64158)
// is not used by any Go code; it is not ported. Go
// `ConfigFileResponse.BuildOptions` (ts#64158) is never set, so it is
// always omitted and not ported.

// Go: proto.go ReleaseSourceFileParams (ts#64434)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReleaseSourceFileParams {
    pub lease: SourceFileLeaseID,
}


// Go: proto.go:897 ProfileParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProfileParams {
    pub dir: String,
}


// Go: proto.go:901 ProfileResult
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProfileResult {
    pub file: String,
}


// Go: proto.go:950 ConfigFileResponse


// Go: proto.go:961 ReadConfigFileResponse
// PORT: Go `Config any` is `CompilerOptionsValue` (see `json_value_to_any`).




// Go: proto.go:966 GetDefaultProjectForFileParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetDefaultProjectForFileParams {
    pub snapshot: SnapshotID,
    pub file: DocumentIdentifier,
}


// Go: proto.go:971 ProjectResponse


// Go: proto.go:983 NewConfigFileResponse
// PORT: Go shares the `*core.CompilerOptions`, `*core.TypeAcquisition` and
// project reference pointers; the response keeps copies (see
// `new_project_response`). Go `Raw.(*collections.OrderedMap[string, any])`
// is `CompilerOptionsValue::Map`.

// Go: proto.go:1011 toProtocolJSONValue

// Go: proto.go:1036 NewProjectResponse
// PORT: Go shares the `*core.CompilerOptions` pointer; the response keeps
// a copy (responses cross into `Box<dyn AnyValue>`, which is `Send`).

// Go: proto.go:1055 GetSymbolAtPositionParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolAtPositionParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub position: u32,
}


// Go: proto.go:1062 GetSymbolsAtPositionsParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolsAtPositionsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub positions: Vec<u32>,
}


// Go: proto.go:1069 GetSymbolOfSourceFileParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolOfSourceFileParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
}


// Go: proto.go:1075 GetSymbolsOfSourceFilesParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolsOfSourceFilesParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub files: Vec<DocumentIdentifier>,
}


// Go: proto.go:1081 GetSymbolAtLocationParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolAtLocationParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle,
}


// Go: proto.go:1087 GetSymbolsAtLocationsParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolsAtLocationsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub locations: Vec<NodeHandle>,
}


// Go: proto.go:1093 SymbolResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SymbolResponse {
    pub id: SymbolID,
    // Project is the project in which the symbol was first observed. It is the
    // default project for follow-up lookups whose results can vary by project.
    pub project: project::ID,
    pub name: String,
    pub flags: u32,
    pub check_flags: u32,
    pub declarations: Vec<NodeHandle>,
    pub value_declaration: NodeHandle,
    pub parent: SymbolID,
    pub export_symbol: SymbolID,
}


// Go: proto.go:1107 symbolHandles

// Go: proto.go:1118 GetTypeOfSymbolParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypeOfSymbolParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub symbol: SymbolID,
}


// Go: proto.go:1124 GetTypesOfSymbolsParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypesOfSymbolsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub symbols: Vec<SymbolID>,
}


// Go: proto.go:1130 TypeResponse


// Go: proto.go:1196 newTypeResponse
// PORT: Go reads the type through its pointer; the port reads it from the
// checker arena that owns `t`.

// Go: proto.go:1283 typeHandles
pub fn type_handles(types: &[TypeId]) -> Vec<TypeID> {
    if types.is_empty() {
        return Vec::new();
    }
    let mut handles = Vec::with_capacity(types.len());
    for &t in types {
        handles.push(type_handle(t));
    }
    handles
}

// Go: proto.go:1294 literalValueToJSON
// PORT: the Go `any` value is `Option<&LiteralValue>` (nil is `None`); the
// result holds the same JSON primitive as `LspAny`.

// Go: proto.go ConstantValueResponse (ts#64241)
// PORT: Go `Value any` holds a JSON primitive (`literalValueToJSON`).


// Go: proto.go:1325 SignatureResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignatureResponse {
    pub id: SignatureID,
    pub flags: u32,
    pub declaration: NodeHandle,
    pub type_parameters: Vec<TypeID>,
    pub parameters: Vec<SymbolID>,
    pub this_parameter: SymbolID,
    pub target: SignatureID,
}


// Go: proto.go:1335 GetSourceFileParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSourceFileParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
}


// Go: proto.go:1341 GetSourceFileNamesParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSourceFileNamesParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
}


// PORT: Go `core.ModuleKind` (and its alias `core.ResolutionMode`) and
// `core.ScriptKind` are Go int32 types with no JSON methods, so JSON uses the
// v2 int arshaler. The params of ts#64247, ts#64292 and ts#64216 decode
// them. Errors name the Go type.
macro_rules! core_int_json {
    ($($ty:ident: $go:literal),* $(,)?) => {$(
        impl MarshalerTo for $ty {
            fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
                self.0.marshal_json_to(enc)
            }
        }

        impl UnmarshalerFrom for $ty {
            fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> {
                self.0
                    .unmarshal_json_from(dec)
                    .map_err(|err| match SemanticError::of(&err) {
                        Some(mut s) => {
                            s.go_type = $go.to_string();
                            s.into_json_error()
                        }
                        None => err,
                    })
            }
        }

        impl IsZero for $ty {
            fn is_zero(&self) -> bool {
                self.0 == 0
            }
        }
    )*};
}


// Go: proto.go GetModeForUsageLocationParams (ts#64292)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetModeForUsageLocationParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub usage: NodeHandle,
}


// Go: proto.go GetModeForResolutionAtIndexParams (ts#64292)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetModeForResolutionAtIndexParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub index: i32,
}


// Go: proto.go GetResolvedModuleParams (ts#64247)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetResolvedModuleParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub module_name: String,
    pub mode: ModuleKind, // Go core.ResolutionMode
}


// Go: proto.go GetResolvedModuleFromModuleSpecifierParams (ts#64247)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetResolvedModuleFromModuleSpecifierParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub module_specifier: NodeHandle,
    pub source_file: Option<DocumentIdentifier>,
}


// Go: proto.go GetResolvedTypeReferenceDirectiveParams (ts#64247)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetResolvedTypeReferenceDirectiveParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub type_directive_name: String,
    pub mode: ModuleKind, // Go core.ResolutionMode
}


// Go: proto.go GetResolvedTypeReferenceDirectiveFromReferenceParams (ts#64247)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetResolvedTypeReferenceDirectiveFromReferenceParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub source_file: DocumentIdentifier,
    pub type_directive_name: String,
    pub resolution_mode: ModuleKind, // Go core.ResolutionMode
}


// Go: proto.go PackageId (ts#64247)
// PORT: `crate::program::PackageId` is Go `module.PackageId`; code outside
// this file names the api one `proto::PackageId`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackageId {
    pub name: String,
    pub sub_module_name: String,
    pub version: String,
    pub peer_dependencies: String,
}


// Go: proto.go NewPackageId (ts#64247)
pub fn new_package_id(package_id: &crate::program::PackageId) -> Option<PackageId> {
    if package_id.name.is_empty() {
        return None;
    }
    Some(PackageId {
        name: package_id.name.clone(),
        sub_module_name: package_id.sub_module_name.clone(),
        version: package_id.version.clone(),
        peer_dependencies: package_id.peer_dependencies.clone(),
    })
}

// Go: proto.go ResolvedModule (ts#64247)
// PORT: `crate::program::ResolvedModule` is Go `module.ResolvedModule`; code
// outside this file names the api one `proto::ResolvedModule`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedModule {
    pub resolved_file_name: String,
    pub original_path: String,
    pub extension: String,
    pub resolved_using_ts_extension: bool,
    pub resolved_using_extra_extensions: bool,
    pub package_id: Option<PackageId>,
    pub is_external_library_import: bool,
    pub alternate_result: String,
}


// Go: proto.go ResolvedTypeReferenceDirective (ts#64247)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedTypeReferenceDirective {
    pub primary: bool,
    pub resolved_file_name: String,
    pub original_path: String,
    pub package_id: Option<PackageId>,
    pub is_external_library_import: bool,
}


// SourceFileMetadata carries program-stored metadata about a single source file.
// Go: proto.go:1430 SourceFileMetadata
// PORT: Go `core.ResolutionMode` marshals as its int32 value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceFileMetadata {
    pub is_default_library: bool,
    pub is_from_external_library: bool,
    pub package_json_type: String,
    pub package_json_directory: String,
    pub implied_node_format: i32,
}


// Go: proto.go:1438 ResolveNameParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolveNameParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub name: String,
    pub location: NodeHandle, // Optional: node handle for location context
    pub file: Option<DocumentIdentifier>, // Optional: file for location context (alternative to Location)
    pub position: Option<u32>, // Optional: position in file for location context (with File)
    pub meaning: u32,          // SymbolFlags for what kind of symbol to find
    pub exclude_globals: bool, // Whether to exclude global symbols
}


// GetSymbolsInScopeParams are parameters for getSymbolsInScope, which returns
// all symbols visible at a given location.
// Go: proto.go:1451 GetSymbolsInScopeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolsInScopeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle, // Optional: node handle for location context
    pub file: Option<DocumentIdentifier>, // Optional: file for location context (alternative to Location)
    pub position: Option<u32>, // Optional: position in file for location context (with File)
    pub meaning: u32,          // SymbolFlags for what kind of symbols to find
}


// GetTypePropertyParams is used for all type sub-property endpoints.
// Go: proto.go:1461 GetTypePropertyParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypePropertyParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
}


// GetSymbolPropertyParams is used for all symbol sub-property endpoints.
// Go: proto.go:1468 GetSymbolPropertyParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSymbolPropertyParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub symbol: SymbolID,
}


// GetSignaturePropertyParams is used for all signature sub-property endpoints.
// Go: proto.go:1475 GetSignaturePropertyParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSignaturePropertyParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub signature: SignatureID,
}


// GetContextualTypeParams returns the contextual type for a node.
// Go: proto.go:1482 GetContextualTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetContextualTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle,
}


// Go: proto.go GetContextualTypeForArgumentParams (ts#64264)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetContextualTypeForArgumentParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle,
    pub index: i32,
}


// GetTypeOfSymbolAtLocationParams returns the narrowed type of a symbol at a specific location.
// Go: proto.go:1496 GetTypeOfSymbolAtLocationParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypeOfSymbolAtLocationParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub symbol: SymbolID,
    pub location: NodeHandle,
}


// GetReferencesToSymbolInFileParams are the parameters for the getReferencesToSymbolInFile method.
// Go: proto.go:1504 GetReferencesToSymbolInFileParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetReferencesToSymbolInFileParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub symbol: SymbolID,
}


// GetReferencedSymbolsForNodeParams are the parameters for the getReferencedSymbolsForNode method.
// Go: proto.go:1512 GetReferencedSymbolsForNodeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetReferencedSymbolsForNodeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub node: NodeHandle,
    pub position: i32,
}


// ReferencedSymbolEntry represents a symbol definition and its references.
// Go: proto.go:1520 ReferencedSymbolEntry
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReferencedSymbolEntry {
    pub definition: NodeHandle,
    pub symbol: Option<SymbolResponse>,
    pub references: Vec<NodeHandle>,
}


// GetSignatureUsagesParams are the parameters for the getSignatureUsages method.
// Go: proto.go:1527 GetSignatureUsagesParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSignatureUsagesParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub signature_decl: NodeHandle,
}


// SignatureUsageResponse represents a single usage of a signature as a name-call pair.
// Go: proto.go:1534 SignatureUsageResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignatureUsageResponse {
    pub name: NodeHandle,
    pub call: NodeHandle,
}


// GetCompletionsAtPositionParams are the parameters for the getCompletionsAtPosition method.
// Go: proto.go:1540 GetCompletionsAtPositionParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetCompletionsAtPositionParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub position: u32,
    pub trigger_character: Option<String>,
    pub include_symbol: bool,
}


// CompletionEntryLabelDetailsResponse holds additional label display text for a completion entry.
// Go: proto.go:1550 CompletionEntryLabelDetailsResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompletionEntryLabelDetailsResponse {
    pub detail: Option<String>,
    pub description: Option<String>,
}


// CompletionEntryResponse represents a single completion item.
// Go: proto.go:1556 CompletionEntryResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompletionEntryResponse {
    pub name: String,
    pub kind: u32,
    pub sort_text: Option<String>,
    pub insert_text: Option<String>,
    pub filter_text: Option<String>,
    pub detail: Option<String>,
    pub label_details: Option<CompletionEntryLabelDetailsResponse>,
    pub symbol: Option<SymbolResponse>,
}


// CompletionInfoResponse wraps a list of completion entries.
// Go: proto.go:1568 CompletionInfoResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompletionInfoResponse {
    pub is_incomplete: bool,
    pub entries: Vec<CompletionEntryResponse>,
}


// GetIntrinsicTypeParams is used for intrinsic type getters (anyType, stringType, etc.).
// Go: proto.go:1574 GetIntrinsicTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetIntrinsicTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
}


// WellKnownSymbolsResponse carries the handle ids of the per-checker singleton
// symbols (unknown, undefined, arguments) so the client can identify them by id
// without a round-trip on every check.
// Go: proto.go:1582 WellKnownSymbolsResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WellKnownSymbolsResponse {
    pub unknown: SymbolID,
    pub undefined: SymbolID,
    pub arguments: SymbolID,
}


// WellKnownSignaturesResponse carries the handle id of the per-checker singleton
// unknown signature (the signature the checker yields when a call cannot be
// resolved) so the client can identify it by id without a round-trip on every check.
// Go: proto.go:1591 WellKnownSignaturesResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WellKnownSignaturesResponse {
    pub unknown: SignatureID,
}


// GetBaseTypeOfLiteralTypeParams returns the base type of a literal type.
// Go: proto.go:1596 GetBaseTypeOfLiteralTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetBaseTypeOfLiteralTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
}


// GetNonNullableTypeParams are the parameters for the getNonNullableType method.
// Go: proto.go:1603 GetNonNullableTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetNonNullableTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
}


// GetTypeFromTypeNodeParams are the parameters for the getTypeFromTypeNode method.
// Go: proto.go:1610 GetTypeFromTypeNodeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypeFromTypeNodeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle,
}


// GetWidenedTypeParams are the parameters for the getWidenedType method.
// Go: proto.go:1617 GetWidenedTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetWidenedTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
}


// GetParameterTypeParams are the parameters for the getParameterType method.
// Go: proto.go:1624 GetParameterTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetParameterTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub signature: SignatureID,
    pub index: i32,
}


// IsArrayLikeTypeParams checks whether a type is array-like.
// Go: proto.go:1632 IsArrayLikeTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IsArrayLikeTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
}


// IsTypeAssignableToParams checks assignability between two types.
// Go: proto.go:1639 IsTypeAssignableToParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IsTypeAssignableToParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub source: TypeID,
    pub target: TypeID,
}


// Go: proto.go:1646 GetSignaturesOfTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetSignaturesOfTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
    pub kind: i32,
}


// Go: proto.go:1653 GetResolvedSignatureParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetResolvedSignatureParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle,
}


// Go: proto.go:1659 GetTypeAtLocationParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypeAtLocationParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle,
}


// Go: proto.go:1665 GetTypeAtLocationsParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypeAtLocationsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub locations: Vec<NodeHandle>,
}


// Go: proto.go:1671 GetTypeAtPositionParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypeAtPositionParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub position: u32,
}


// Go: proto.go:1678 GetTypesAtPositionsParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetTypesAtPositionsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub positions: Vec<u32>,
}


// Go: proto.go:1685 ImportAdderActionKind (tsgo#3881)
// PORT: a Go string type, a newtype as the handles are (see `handle_json!`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImportAdderActionKind(pub String);


// Go: proto.go:1688 ImportAdderActionKindImportSymbol (tsgo#3881)
pub const IMPORT_ADDER_ACTION_KIND_IMPORT_SYMBOL: &str = "importSymbol";

// Go: proto.go:1691 ImportAdderAction (tsgo#3881)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImportAdderAction {
    pub kind: ImportAdderActionKind,
    pub symbol: SymbolID,
    pub is_valid_type_only_use_site: Option<bool>,
}


// Go: proto.go:1697 GetImportAdderEditsParams (tsgo#3881)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetImportAdderEditsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier,
    pub actions: Vec<ImportAdderAction>,
}


// Go: proto.go:1704 TextEdit (tsgo#3881)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextEdit {
    pub pos: i32,
    pub end: i32,
    pub new_text: String,
}


// TypeToTypeNodeParams are the parameters for the typeToTypeNode method.
// Go: proto.go:1711 TypeToTypeNodeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeToTypeNodeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
    pub location: NodeHandle,
    pub flags: i32,
}


// SignatureToSignatureDeclarationParams are the parameters for the signatureToSignatureDeclaration method.
// Go: proto.go:1720 SignatureToSignatureDeclarationParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignatureToSignatureDeclarationParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub signature: SignatureID,
    pub kind: i32,
    pub location: NodeHandle,
    pub flags: i32,
}


// PrintNodeParams are the parameters for the printNode method.
// Go: proto.go:1730 PrintNodeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrintNodeParams {
    pub data: String, // base64-encoded binary AST data
    pub preserve_source_newlines: bool,
    pub never_ascii_escape: bool,
    pub terminate_unterminated_literals: bool,
}


// Go: proto.go:1737 EmitParams (tsgo#4699)
// PORT: Go `*uint32` is `Option<u32>`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmitParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub emit_only: Option<u32>,
}


// Go: proto.go:1743 SelectedFilesEmitParams (tsgo#4699)
// PORT: Go tells a nil `Files` (absent or `null`) from an empty one, so it
// is `Option<Vec>`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectedFilesEmitParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub files: Option<Vec<DocumentIdentifier>>,
}


// Go: proto.go:1749 EmitResponse (tsgo#4699)


// Go: proto.go:1758 EmitOutputFile (tsgo#4699)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmitOutputFile {
    pub file_name: String,
    pub text: String,
    pub source_file_name: Option<String>,
}


// Go: proto.go:1764 EmitOutputResponse (tsgo#4699)


// FormatNodeForInsertionParams are the parameters for the formatNodeForInsertion method.
// Go: proto.go:1771 FormatNodeForInsertionParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FormatNodeForInsertionParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub file: DocumentIdentifier, // target file where the node will be inserted
    pub position: u32, // UTF-16 code-unit offset of the insertion position in the target file
    pub data: String,  // base64-encoded binary AST data for the synthesized node
}


// CheckerTypeParams are parameters for checker methods that operate on a type.
// Go: proto.go:1780 CheckerTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckerTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
}


// GetPropertyOfTypeParams are parameters for getPropertyOfType (a named property of a type).
// Go: proto.go:1787 GetPropertyOfTypeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetPropertyOfTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
    pub name: String,
}


// Go: proto.go GetIndexInfoOfTypeParams (ts#64264)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetIndexInfoOfTypeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub type_: TypeID,
    pub kind: i32,
}


// CheckerNodeParams are parameters for checker methods that operate on a node location.
// Go: proto.go:1810 CheckerNodeParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckerNodeParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub location: NodeHandle,
}


// GetMemberInModuleExportsParams are parameters for getMemberInModuleExports.
// Go: proto.go:1802 GetMemberInModuleExportsParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetMemberInModuleExportsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub symbol: SymbolID,
    pub name: String,
}


// CheckerSymbolParams are parameters for checker methods that operate on a symbol.
// Go: proto.go:1817 CheckerSymbolParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckerSymbolParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub symbol: SymbolID,
}


// JSDocTagInfo is a single JSDoc tag, mirroring Strada's JSDocTagInfo but with the tag text
// rendered as a plain string rather than SymbolDisplayPart[].
// Go: proto.go:1825 JSDocTagInfo
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: String,
}


// CheckerSignatureParams are parameters for checker methods that operate on a signature.
// Go: proto.go:1831 CheckerSignatureParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckerSignatureParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub signature: SignatureID,
}


// TypePredicateResponse is the response for getTypePredicateOfSignature.
// Go: proto.go:1838 TypePredicateResponse


// IndexInfoResponse represents a single index signature.
// Go: proto.go:1846 IndexInfoResponse


// SourceFileResponse contains the binary-encoded AST data for a source file.
// The Data field is base64-encoded binary data in the encoder's format.
// Go: proto.go:1855 SourceFileResponse
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceFileResponse {
    // Data is the base64-encoded binary AST data in the encoder's format.
    pub data: String,
}

// Go: the v2 default struct marshal (`data` is `plain`).
// PERF: (apiperf1) `data` is base64 text, which has no byte that JSON
// escapes, so `PlainJsonText` copies it as it is. The string marshal of
// each char was 12% of an API createSourceFile loop of a 260 KB file.

/// PERF: (apiperf1) the string marshal of a text that may need no JSON
/// escape (`SourceFileResponse.data`). When every byte is ASCII and none is
/// a control character, `"` or `\`, the string marshal writes the text as
/// it is between quotes, so this copies it at once. Other text takes the
/// string marshal.
struct PlainJsonText<'a>(&'a str);


// GetDiagnosticsParams are parameters for per-file diagnostic methods.
// Go: proto.go:1861 GetDiagnosticsParams
#[derive(Clone, Debug, Default, PartialEq)]
// PORT: Go `Files []DocumentIdentifier` is `Option`: `None` is a nil slice
// (no `files`, or `null`) and `Some(vec![])` is `[]`. `getDiagnostics`
// tells them apart.
pub struct GetDiagnosticsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
    pub files: Option<Vec<DocumentIdentifier>>,
}


// GetProjectDiagnosticsParams are parameters for project-wide diagnostic methods.
// Go: proto.go:1868 GetProjectDiagnosticsParams
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GetProjectDiagnosticsParams {
    pub snapshot: SnapshotID,
    pub project: project::ID,
}


// DiagnosticResponse is the API response for a single diagnostic.
// Go: proto.go:1874 DiagnosticResponse
// PORT: `Default` is written below; `diagnostics.Category` has none in the
// port.


// PORT: Go `diagnostics.Category` is a Go int32 type; the Go zero value is
// `CategoryWarning`.

// PORT: the Go v2 default struct unmarshal of `DiagnosticResponse`
// (`CreateProgramOptions.ConfigFileParsingDiagnostics`, ts#63950).

// Go: proto.go DiagnosticResponse.ToDiagnostic (ts#63950)

// Go: proto.go DiagnosticPositionResponse (ts#63935)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiagnosticPositionResponse {
    pub line: i32,
    // PORT: Go `core.UTF16Offset`.
    pub character: i32,
}


// Go: proto.go DiagnosticSourceLineResponse (ts#63935)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiagnosticSourceLineResponse {
    pub line: i32,
    pub text: String,
}


// Go: proto.go diagnosticSourceLines (ts#63935)
// PORT: Go returns nil for an empty line map; the field is `omitempty`, so
// an empty `Vec` writes the same JSON.

// Go: proto.go:1944 NewDiagnosticResponse
// NewDiagnosticResponse converts an ast.Diagnostic to a DiagnosticResponse.

// Go: proto.go newDiagnosticResponse (ts#63935)
// PORT: Go names it `newDiagnosticResponse`, which snakes to the name of the
// exported function; the wrapped form gets the `_wrapped` suffix.

// Go: proto.go:2015 NewDiagnosticResponses
// NewDiagnosticResponses converts a slice of ast.Diagnostics to DiagnosticResponses.

// Go: proto.go:2026 unmarshalPayload

// Go: proto.go:2034 unmarshallerFor
// `unmarshal_root` is Go `json.Unmarshal` with the v2 error text.

// Go: proto.go:2042 noParams

// PORT: the JSON form of `core.CompilerOptions` (`CompilerOptionsJSON`,
// `TypeAcquisitionJSON` and the `CompilerOptions` decode) and
// `marshal_field_omitempty` live in `crate::options_json`, so the compiler
// side (content mappers) does not depend on `api`. Go keeps them in `core`
// (struct tags) and in the json package.
pub use crate::options_json::{TypeAcquisitionJSON, };

#[cfg(test)]
mod source_file_response_tests {
    use super::*;
    use crate::frontend::json::json_marshal;

    // apiperf1: the marshal of `SourceFileResponse` is the v2 struct marshal
    // of its string, for base64 text (copied as it is) and for text with
    // bytes that JSON escapes or that are not ASCII (the string marshal).
    #[test]
    fn data_is_the_string_marshal() {
        for data in ["", "Zm9vYmE=", "ab+/09==", "a\"b\\c\n\u{1}", "\u{7f}é"] {
            let response = SourceFileResponse {
                data: data.to_string(),
            };
            let want = format!(
                "{{\"data\":{}}}",
                json_marshal(&data.to_string(), &[]).expect("a string marshals")
            );
            assert_eq!(json_marshal(&response, &[]).expect("it marshals"), want);
        }
    }
}

#[cfg(test)]
mod config_file_response_tests {
    use super::*;
    use crate::frontend::tsoptions::new_parsed_command_line;
    use crate::frontend::tspath::ComparePathsOptions;
    use std::rc::Rc;

    // Go NewConfigFileResponse reads `p.Errors` (api/proto.go:996), where
    // checkSourceFilesBelongToPath (tsoptions/parsedcommandline.go:181)
    // appended TS6059 when the project's program read
    // `CommonSourceDirectory` (projfix1 skeptic: a referenced project with
    // outDir, `rootDir: src` and a file outside src gives
    // `parsedCommandLine.errors: [TS6059]` in Go N's createSnapshot).
    #[test]
    fn errors_hold_the_common_source_directory_errors_like_go() {
        let command_line = new_parsed_command_line(
            Rc::new(CompilerOptions {
                root_dir: "/p/src".to_string(),
                out_dir: "/p/out".to_string(),
                ..Default::default()
            }),
            vec!["/p/src/a.ts".to_string(), "/p/extra.ts".to_string()],
            None,
            ComparePathsOptions {
                use_case_sensitive_file_names: true,
                current_directory: "/p".to_string(),
            },
        );
        let codes = |command_line| {
            new_config_file_response(Some(command_line))
                .expect("a response")
                .errors
                .iter()
                .map(|error| error.code)
                .collect::<Vec<_>>()
        };
        assert!(codes(&command_line).is_empty());
        assert_eq!(command_line.common_source_directory(), "/p/src/");
        assert_eq!(codes(&command_line), [6059]);
    }
}

#[cfg(test)]
mod unmarshal_error_tests {
    use super::*;

    fn err_text<T: UnmarshalerFrom + Default + AnyValue>(data: &str) -> String {
        unmarshaller_for::<T>(data.as_bytes())
            .expect_err("want an error")
            .error()
    }

    // Expected texts come from the pinned Go API (tests2 S5-003 goldens and
    // Go JSON v2); Go writes "cannot" or "unable to".
    #[test]
    fn decode_errors_match_go() {
        assert_eq!(
            err_text::<GetSymbolAtPositionParams>(
                r#"{"snapshot":1,"project":"p","file":"a.ts","position":"x"}"#
            ),
            r#"failed to unmarshal *api.GetSymbolAtPositionParams: json: cannot unmarshal JSON string into Go uint32 within "/position""#
        );
        assert_eq!(
            err_text::<ReleaseParams>(r#"{"snapshot":"x"}"#),
            r#"failed to unmarshal *api.ReleaseParams: json: cannot unmarshal JSON string into Go api.SnapshotID within "/snapshot""#
        );
        assert_eq!(
            err_text::<GetSymbolsAtPositionsParams>(r#"{"snapshot":1,"positions":[1,"x"]}"#),
            r#"failed to unmarshal *api.GetSymbolsAtPositionsParams: json: cannot unmarshal JSON string into Go uint32 within "/positions/1""#
        );
        assert_eq!(
            err_text::<GetSourceFileParams>(r#"{"snapshot":1,"file":5}"#),
            r#"failed to unmarshal *api.GetSourceFileParams: json: cannot unmarshal into Go api.DocumentIdentifier within "/file": DocumentIdentifier: expected string or object, got number"#
        );
    }

    // Go `diagnostics.Category` is `type Category int32`
    // (diagnostics/diagnostics.go:18): a config file parsing diagnostic that
    // the client sends keeps any int32 category, and a number out of range
    // names that type. The error text is Go N's for the same
    // `updateSnapshot` request.
    #[test]
    fn a_sent_diagnostic_category_is_any_int32() {
        let decode = |category: &str| {
            let data = format!(
                r#"{{"configFileParsingDiagnostics":[{{"code":1,"category":{category},"text":"x"}}]}}"#
            );
            let mut options = CreateProgramOptions::default();
            crate::frontend::json_ext::unmarshal_root(data.as_bytes(), &mut options)
                .map(|()| options.config_file_parsing_diagnostics[0].category)
                .map_err(|err| err.message)
        };
        assert_eq!(decode("1"), Ok(crate::diagnostics::Category::Error));
        assert_eq!(decode("999"), Ok(crate::diagnostics::Category(999)));
        assert_eq!(
            decode("-2147483648"),
            Ok(crate::diagnostics::Category(i32::MIN))
        );
        assert_eq!(
            decode("3000000000"),
            Err(r#"json: cannot unmarshal JSON number 3000000000 into Go diagnostics.Category within "/configFileParsingDiagnostics/0/category": value out of range"#.to_string())
        );
    }

    // tsgo#4849: the client sends back the `compilerOptions` that the API
    // wrote (`CompilerOptionsJSON`). Decoding them and writing them again
    // gives the same text. The options are those of the hono-ext traces.
    #[test]
    fn transpile_compiler_options_round_trip() {
        let options = r#"{"composite":true,"declaration":true,"forceConsistentCasingInFileNames":true,"module":6,"moduleResolution":100,"noUnusedLocals":true,"noUnusedParameters":true,"outDir":"/p/dist/types","paths":{"a/*":["b/*"],"c":["d"]},"rootDir":"/p/src","skipLibCheck":true,"strict":true,"target":9,"types":["node"],"esModuleInterop":true,"configFilePath":"/p/tsconfig.build.json"}"#;
        let data = format!(
            r#"{{"fileName":"/p/src/a.ts","options":{{"compilerOptions":{options},"reportDiagnostics":true}}}}"#
        );
        let parsed = unmarshaller_for::<TranspileFromFileParams>(data.as_bytes())
            .expect("valid params")
            .expect("params value");
        let params = parsed
            .downcast_ref::<TranspileFromFileParams>()
            .expect("TranspileFromFileParams");
        assert!(params.options.report_diagnostics);
        let compiler_options = params
            .options
            .compiler_options
            .as_ref()
            .expect("compilerOptions");
        assert_eq!(compiler_options.module, ModuleKind(6));
        assert_eq!(compiler_options.strict, Tristate::True);
        let written =
            crate::frontend::json::json_marshal(&CompilerOptionsJSON(compiler_options), &[])
                .expect("marshal");
        assert_eq!(written, options);
    }
}
