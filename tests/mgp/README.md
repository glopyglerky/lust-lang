# Minimal generic pipeline (MGP) fixtures

The ARCH 9.3 MGP is a thin end-to-end slice over local types
only: one generic Adt, one trait with an input parameter (the
whole trait ref, not Self alone), one associated equality
used in a signature, one generic method call (inherent and
trait), one drop of a generic value, and one trait that has
both a concrete impl and a bounded bare-param blanket impl,
selected through a `for` loop over a local iterator type.

The slice is split into small programs so a failure points at
one shape:

- `pass/mgp_adt_trait.rs`: generic Adt, the input-parameter
  trait, the associated equality, and both method calls.
- `pass/mgp_n1_for.rs`: the concrete-plus-blanket selection
  through a `for` loop.
- `pass/mgp_drop_generic.rs`: the drop of a generic value.

Each program's stdout must equal its sibling `.stdout`. The
runner is `scripts/test-traits.sh`. Per ARCH 9.3 the MGP gate
itself also builds and runs through the daemon; that part is
W-DRIVER's gate, not this corpus's.
