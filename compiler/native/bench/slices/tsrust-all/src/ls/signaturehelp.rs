use crate::ls::prelude::*;

use crate::spanmap::Feature;

// Go: ls/signaturehelp.go:25 SignatureHelpTriggerCharacters
// SignatureHelpTriggerCharacters and SignatureHelpRetriggerCharacters are the characters that trigger and
// re-trigger signature help. They are advertised both in the static server capabilities and in the dynamic
// content-mapper registration, so they live here to keep those two declarations in sync.
// PORT: Go `[]string` package variables; here constant slices.
pub const SIGNATURE_HELP_TRIGGER_CHARACTERS: &[&str] = &["(", ",", "<"];
// Go: ls/signaturehelp.go:26 SignatureHelpRetriggerCharacters
pub const SIGNATURE_HELP_RETRIGGER_CHARACTERS: &[&str] = &[")"];

// Go: ls/signaturehelp.go:29 callInvocation
#[derive(Clone, Copy, Debug)]
pub struct CallInvocation {
    pub node: Node,
}

// Go: ls/signaturehelp.go:33 typeArgsInvocation
#[derive(Clone, Copy, Debug)]
pub struct TypeArgsInvocation {
    pub called: Node,
}

// Go: ls/signaturehelp.go:37 contextualInvocation
#[derive(Clone, Copy, Debug)]
pub struct ContextualInvocation {
    pub signature: SignatureId,
    pub node: Node, // Just for enclosingDeclaration for printing types
    pub symbol: SymbolId,
}

// Go: ls/signaturehelp.go:43 invocation
// PORT: Go `*callInvocation` (and the other two) are nil-able pointers; here `Option`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Invocation {
    pub call_invocation: Option<CallInvocation>,
    pub type_args_invocation: Option<TypeArgsInvocation>,
    pub contextual_invocation: Option<ContextualInvocation>,
}

// Go: ls/signaturehelp.go:76 signatureHelpTriggerReasonKind (local type of GetSignatureHelpItems)
const SIGNATURE_HELP_TRIGGER_REASON_KIND_NONE: i32 = 0; // was undefined
const SIGNATURE_HELP_TRIGGER_REASON_KIND_INVOKED: i32 = 1; // was "invoked"
const SIGNATURE_HELP_TRIGGER_REASON_KIND_CHARACTER_TYPED: i32 = 2; // was "characterTyped"
const SIGNATURE_HELP_TRIGGER_REASON_KIND_RETRIGGERED: i32 = 3; // was "retrigger"


// Go: ls/signaturehelp.go:169 createTypeHelpItems

// Go: ls/signaturehelp.go:211 getTypeHelpItem


// Go: ls/signaturehelp.go:488 returnTypeToDisplayParts


// Go: ls/signaturehelp.go:666 signatureHelpNodeBuilderFlags


// Go: ls/signaturehelp.go:699 createSignatureHelpParameterForTypeParameter

// Go: ls/signaturehelp.go:713 signatureInformation
// Represents the signature of something callable. A signature
// can have a label, like a function-name, a doc-comment, and
// a set of parameters.

// Go: ls/signaturehelp.go:728 signatureHelpItemInfo

// Go: ls/signaturehelp.go:734 signatureHelpParameter
// PORT: Go shares the `*lsproto.ParameterInformation` pointer between copies;
// nothing mutates it after creation, so a value is used.

// Go: ls/signaturehelp.go:740 getEnclosingDeclarationFromInvocation
pub fn get_enclosing_declaration_from_invocation(invocation: &Invocation) -> Node {
    if let Some(call_invocation) = &invocation.call_invocation {
        call_invocation.node
    } else if let Some(type_args_invocation) = &invocation.type_args_invocation {
        type_args_invocation.called
    } else {
        invocation
            .contextual_invocation
            .as_ref()
            .unwrap_or_else(|| crate::core::go_nil_dereference())
            .node
    }
}

// Go: ls/signaturehelp.go:750 getExpressionFromInvocation

// Go: ls/signaturehelp.go:757 candidateInfo
#[derive(Clone, Debug, Default)]
pub struct CandidateInfo {
    pub candidates: Vec<SignatureId>,
    pub resolved_signature: SignatureId,
}

// Go: ls/signaturehelp.go:762 CandidateOrTypeInfo
#[derive(Clone, Debug, Default)]
pub struct CandidateOrTypeInfo {
    pub candidate_info: Option<CandidateInfo>,
    pub type_info: SymbolId,
}

// Go: ls/signaturehelp.go:767 getCandidateOrTypeInfo

// Go: ls/signaturehelp.go:828 isSyntacticOwner

// Go: ls/signaturehelp.go:843 containsPrecedingToken

// Go: ls/signaturehelp.go:861 getContainingArgumentInfo

// Go: ls/signaturehelp.go:904 getImmediatelyContainingArgumentOrContextualParameterInfo

// Go: ls/signaturehelp.go:912 argumentListInfo
// PORT: Go `*invocation` is a pointer that nothing mutates after creation; a value here.
#[derive(Clone, Copy, Debug)]
pub struct ArgumentListInfo {
    pub is_type_parameter_list: bool,
    pub invocation: Invocation,
    pub arguments_span: TextRange,
    pub argument_index: i32,
    /** argumentCount is the *apparent* number of arguments. */
    pub argument_count: i32,
}

// Go: ls/signaturehelp.go:923 getImmediatelyContainingArgumentInfo
// Returns relevant information for the argument list and the current argument if we are
// in the argument of an invocation; returns undefined otherwise.

// Go: ls/signaturehelp.go:1029 getArgumentIndexForTemplatePiece
// spanIndex is either the index for a given template span.
// This does not give appropriate results for a NoSubstitutionTemplateLiteral

// Go: ls/signaturehelp.go:1051 getAdjustedNode

// Go: ls/signaturehelp.go:1067 contextualSignatureLocationInfo
#[derive(Clone, Copy, Debug)]
pub struct ContextualSignatureLocationInfo {
    pub contextual_type: TypeId,
    pub argument_index: i32,
    pub argument_count: i32,
    pub arguments_span: TextRange,
}

// Go: ls/signaturehelp.go:1074 getSpreadElementCount

// Go: ls/signaturehelp.go:1098 getArgumentIndex

// Go: ls/signaturehelp.go:1102 getArgumentCount

// Go: ls/signaturehelp.go:1106 getArgumentIndexOrCount

// Go: ls/signaturehelp.go:1148 argumentOrParameterListInfo

// Go: ls/signaturehelp.go:1155 getArgumentOrParameterListInfo

// Go: ls/signaturehelp.go:1172 getApplicableSpanForArguments

// Go: ls/signaturehelp.go:1204 ensureMinimumSpanSize
// ensureMinimumSpanSize ensures that a span includes at least one position.
// TextRange.Contains uses a half-open interval, so an empty span would not contain
// the cursor immediately after typing an opening paren in a call like foo(bar(|)).
pub fn ensure_minimum_span_size(start: i32, end: i32) -> i32 {
    if end <= start {
        return start + 1;
    }
    end
}

// Go: ls/signaturehelp.go:1211 argumentOrParameterListAndIndex

// Go: ls/signaturehelp.go:1216 getArgumentOrParameterListAndIndex

// Go: ls/signaturehelp.go:1244 getChildListThatStartsWithOpenerToken

// Go: ls/signaturehelp.go:1261 tryGetParameterInfo

// Go: ls/signaturehelp.go:1302 chooseBetterSymbol
// PORT: symbol data lives in the checker arena, so the arena is the first parameter.

// Go: ls/signaturehelp.go:1313 getContextualSignatureLocationInfo

// Go: ls/signaturehelp.go:1361 getHighestBinary

// Go: ls/signaturehelp.go:1368 countBinaryExpressionParameters

// Go: ls/signaturehelp.go:1375 getTokenFromNodeList

// Go: ls/signaturehelp.go:1399 getArgumentListInfoForTemplate

// Go: ls/signaturehelp.go:1417 getApplicableRangeForTaggedTemplate
