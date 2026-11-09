use crate::format::prelude::*;

// Go: format/util.go:10 rangeIsOnOneLine

// Go: format/util.go:16 getOpenTokenForList

// Go: format/util.go:63 getCloseTokenForOpenToken
pub fn get_close_token_for_open_token(kind: SyntaxKind) -> SyntaxKind {
    // TODO: matches strada - seems like it could handle more pairs of braces, though? [] notably missing
    match kind {
        SyntaxKind::OpenParenToken => return SyntaxKind::CloseParenToken,
        SyntaxKind::LessThanToken => return SyntaxKind::GreaterThanToken,
        SyntaxKind::OpenBraceToken => return SyntaxKind::CloseBraceToken,
        _ => {}
    }
    SyntaxKind::Unknown
}

// Go: format/util.go:76 GetLineStartPositionForPosition



// Returns true if node is a element in some list in parent
// i.e. parent is class declaration with the list of members and node is one of members.
// Go: format/util.go:121 isListElement

// Go: format/util.go:137 isMemberListElement
