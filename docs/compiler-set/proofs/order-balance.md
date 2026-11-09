# Execution-position balance

Future runner reports identify their schedule as
`order_method = "randomized-cyclic-blocks-v1"`. This changes experiment order,
not compiler behavior or the paired-log estimator. The published
[2026-11-09 measurements](../measurements/2026-11-09-shared-v2/README.md) used
the earlier forward/reverse schedule and keep their original data and limits.

## Construction

For `L` lanes, each block shuffles the lane labels `pi` and separately shuffles
the shifts `rho = [0, ..., L-1]`. A row for shift `s` places lane
`pi[(p + s) mod L]` at execution position `p`. It executes every lane once.
The block visits each shift once, stopping early only if the phase has fewer
rounds remaining. Each subsequent block chooses new permutations.

The implementation uses seeded SplitMix64, rejection sampling for bounded
indices, and Fisher-Yates shuffles. The seed and phase lengths reproduce the
recorded order. Warmup and timed phases start new blocks, sharing one continued
generator state. Warmup records remain excluded from statistics. The construction
does not assert that this deterministic generator supplies independent random
variables on a real host.

## Position theorem

Assume each block's label and shift arrays are permutations of `0..L`.
Both implementation arrays start as `0..L`. Fisher-Yates only swaps elements,
so it preserves that property. Its bounded index is a remainder modulo a
positive bound and therefore selects a valid swap position.
For a lane `l`, write `j = pi^-1(l)`. Its position in shift `s` is
`p = (j - s) mod L`. Subtraction modulo `L` is a bijection. Thus, as a complete
block visits all shifts, that lane visits every position exactly once.
The same holds for every lane and every choice of the two permutations.

After `R = qL + r` rounds, with `0 <= r < L`, the `q` complete blocks contribute
exactly `q` occurrences of each lane at each position. The remaining `r` shifts
are distinct, so a particular lane visits any position at most once more.
Every lane-position count is therefore either `q` or `q+1`. Counts for that
lane differ by at most one. If `L` divides `R`, every count is exactly `R/L`.
This theorem is deterministic; it needs no bootstrap or timing assumptions.

For example, three lanes and eight rounds give each lane two or three visits
to every position. Twelve rounds give four visits to every position. A lane
can no longer occupy the middle position for all eight rounds, as N1 did in
the earlier three-lane forward/reverse experiment.

Unit checks enumerate all prefixes through four blocks plus one round for
2-8 lanes and four seeds, including the boundary generator seed. They verify
row permutations, the exact complete-block counts, the prefix bound and
reproducibility. The real three-lane fixture checks the emitted report's method,
24 timed records and position counts. These executable checks support the
implementation; the bijection argument supplies the general position theorem.
No proof assistant checked this implementation.

## What this does not establish

Position balance does not balance every immediate predecessor, every ordered
lane pair, or all permutations. It does not remove cache, thermal, scheduler,
background-load or time-trend effects. Incomplete blocks have the stated bound,
not exact equality. Choose a round count divisible by the lane count, satisfying
the runner's existing even-count and minimum-eight rules, for exact balance.

Neither position balance nor pseudo-random ordering proves equivalent compiler
work, independent representative pairs, null sign exchangeability, bootstrap
coverage, runtime correctness or a speedup. Correctness gates must still use
the configuration they intend to qualify. A direct Cargo test without an
explicit frontend option does not qualify that option merely because a timed
command used it. Those assumptions and gate limitations remain separate from
this theorem. See the [statistical analysis](benchmark-analysis.md).
