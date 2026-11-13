// Go: internal/rules/obsolete_match_import.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct ObsoleteMatchImportMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub imported_specifier: String,
}

// Go: rules/obsolete_match_import.go AnalyzeObsoleteMatchImport
