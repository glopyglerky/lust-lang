//! Port of Go `ls/displaypartswriter.go`.

use crate::ls::prelude::*;

use crate::frontend::scanner::scanner_p1::{RUNE_ERROR, };

// Go: ls/displaypartswriter.go:14 `var _ printer.EmitTextWriter = &displayPartsWriter{}`
// PORT: the `impl EmitTextWriter for DisplayPartsWriter` below.

// Go: ls/displaypartswriter.go:19 displayPartsWriter
// displayPartsWriter implements EmitTextWriter and captures classified text runs
// for VS colorized labels, while also building a plain string.
// When vsCapability is false, only the plain string is built; runs are skipped.
//
// PORT: Go shares `*displayPartsWriter` between its builder and the printer
// (`p.Write(node, file, w, nil)`), so the handle is
// `Rc<RefCell<DisplayPartsWriter>>` (`new_display_parts_writer`). It coerces
// to the printer's `Rc<RefCell<dyn EmitTextWriter>>`.
//
// PORT: `WriteSymbol` classifies the run from the symbol flags
// (`classificationForSymbol`). The printer calls it while the caller holds
// the checker, and the flags live in the checker's symbol arena. So
// `write_symbol` adds the run with an empty classification and records the
// run index and the symbol in `symbol_runs`; `get_runs(c)` fills in the
// classification with the checker. The plain string does not depend on the
// classification. The runs equal Go's when the symbol flags and the first
// declaration do not change between printing and `get_runs`; the checker
// sets both when it makes or merges a symbol, before the language service
// prints it.
#[derive(Clone, Debug, Default)]
pub struct DisplayPartsWriter {
    pub builder: String,
    pub runs: Vec<lsproto::VSClassifiedTextRun>,
    pub vs_capability: bool,
    pub last_written: String,
    /// PORT: runs made by `write_symbol` whose classification `get_runs`
    /// computes: `(index in runs, symbol)`.
    pub symbol_runs: Vec<(usize, SymbolId)>,
}

// Go: ls/displaypartswriter.go:26 newDisplayPartsWriter
pub fn new_display_parts_writer(vs_capability: bool) -> Rc<RefCell<DisplayPartsWriter>> {
    Rc::new(RefCell::new(DisplayPartsWriter {
        vs_capability,
        ..Default::default()
    }))
}

impl DisplayPartsWriter {
    // Go: ls/displaypartswriter.go:30 addRun
    pub fn add_run(&mut self, classification: lsproto::ClassificationTypeName, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.vs_capability {
            self.runs.push(lsproto::VSClassifiedTextRun {
                classification_type_name: classification.0.to_string(),
                text: text.to_string(),
                ..Default::default()
            });
        }
        self.last_written = text.to_string();
        self.builder.push_str(text);
    }

    // Go: ls/displaypartswriter.go:45 WriteClassified
    // WriteClassified writes text with an explicit classification type.
    pub fn write_classified(
        &mut self,
        text: &str,
        classification: lsproto::ClassificationTypeName,
    ) {
        self.add_run(classification, text);
    }

    // Go: ls/displaypartswriter.go:50 WriteFrom
    // WriteFrom copies the accumulated content from another displayPartsWriter.
    // PORT: Go appends `other.GetRuns()`; the runs of `other` that still wait
    // for their symbol classification move along with their new index.

    // Go: ls/displaypartswriter.go:60 GetRuns
    // PORT: takes the checker to classify the `write_symbol` runs (see the
    // struct comment). Go returns the shared run pointers; this returns
    // copies.
}


// Go: ls/displaypartswriter.go:168 classificationForSymbol
// classificationForSymbol determines the Roslyn classification type name based on a symbol's flags.
// Matches the Strada translation chain: displayPartKind() → GetClassificationName().
// PORT: Go reads the symbol through its pointer; `symbols` is the arena that
// holds it (the checker's).

// Go: ls/displaypartswriter.go:211 isFirstDeclarationOfSymbolParameter
// isFirstDeclarationOfSymbolParameter checks if the symbol's first declaration is a parameter.
