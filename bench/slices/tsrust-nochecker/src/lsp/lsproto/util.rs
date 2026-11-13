//! Port of internal/lsp/lsproto/util.go.


// Go: util.go:11 ComparePositions
// Implements a cmp.Compare like function for two Position
// ComparePositions(pos, other) == cmp.Compare(pos, other)

// Go: util.go:22 CompareRanges
// Implements a cmp.Compare like function for two Range
// CompareRanges(lsRange, other) == cmp.Compare(lsRange, other)
//
//	Range.Start is compared before Range.End



// Go: util.go:53 diagnosticExistsInSlice

// Go: util.go:62 diagnosticsEqual

// Go: util.go:69 diagnosticCodesEqual
// PORT: a nil `*IntegerOrString` is a Go nil pointer dereference where Go
// reads it: `code1` first, `code2` only after a `code1` field is set.

// Go: util.go:79 diagnosticMessagesEqual

// Go: util.go:89 CompareDiagnostics
// PORT: Go takes and returns `[]*Diagnostic`; the results borrow from the inputs.

