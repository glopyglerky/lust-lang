# Mathematical arguments for the CGU16 and ThinLTO study

These arguments describe the fixed measurement and decision. They do not prove
the compiler transformation correct or establish unmeasured performance.
See the [dataset](../measurements/2026-11-09-cgu16-thinlto-v3/README.md).

## Paired geometric ratio

For positive candidate and control times `a[c,j]` and `b[c,j]`, define
`x[c,j] = ln(a[c,j]) - ln(b[c,j])`, with three cases and twelve pairs per case.
The aggregate is

```text
theta = (1/3) sum_c (1/12) sum_j x[c,j]
R     = exp(theta) = product_(c,j) (a[c,j]/b[c,j])^(1/36).
```

Proof: logarithms turn multiplication into addition, and exponentiation reverses
that operation. Equal case counts make the equal-case weighting also equal-pair
weighting here. With unequal counts these two estimands differ. Multiplying both
times of any pair by the same positive scale leaves its log difference unchanged.
Reversing numerator and denominator negates every log difference, giving `1/R`.
These are identities over real arithmetic; implementation uses finite f64 values
with explicit validation and a disclosed replay rounding tolerance.

`1-R` is fractional time reduction for this geometric estimand. `1/R` is its
corresponding speed ratio. They are different quantities. The observed `R` is
0.8209119403216099, or 17.9% less time and about 1.218 times the speed of the matched
control in this workload set. It says nothing about a differently weighted
population or the version-confounded stock reference.

## Stratified paired resampling and inversion

Each of 10,000 bootstrap replicates draws twelve pair indices with replacement
within each fixed case, averages each stratum's paired logs, averages the three
means and exponentiates. SplitMix64 seed 6757 continues across strata and
replicates; rejection indexing avoids modulo bias under the generator model.
No observations are omitted. Sorted replicate indices 249 and 9750 implement
the prescribed floor/ceil percentile rule for `B=10000`.

For the same resampled indices, reversing ratios negates each replicate log mean.
Since reciprocal is decreasing on positive numbers, sorting reverses the order.
The endpoint ranks sum to `B-1`, so a ratio interval `[L,U]` maps to `[1/U,1/L]`.
This is an ordering identity, not a proof that bootstrap coverage is exactly 95%.
Population interpretation still assumes representative independent pairs; shared
host effects may violate that assumption. The sign-flip argument separately
requires exchangeability under the null. No aggregate p-value was specified.

## Position counts

A complete cyclic block of three lane permutations places each lane once in
each execution position. Twelve timed rounds contain four complete blocks, so
each lane occupies every position exactly four times. This follows by summing
the one-per-block counts. The recorded seed/order and actual record sequence are
checked by the replay. Equal position counts constrain one scheduling imbalance;
they prove neither independence nor removal of cache/carryover effects.

## Exact RSS decision and its limit

For positive integer control and candidate peaks `m` and `n`,

```text
n/m <= 1.20   iff   5*n <= 6*m.
```

Proof: multiply by the positive `5*m`. The implementation promotes i64 values
to i128 before multiplying, avoiding i64 product overflow. All 36 matched timed
pairs must pass; missing, zero or unbound measurements cannot pass. No floating
threshold rounding is used for this gate.

The measurement is the largest waited process peak. It cannot bound simultaneous
aggregate memory by that peak: two processes can each use `m` concurrently while
their aggregate is `2*m`; the construction extends to any process count. Thus the
observed RSS gate cannot establish an aggregate-RAM limit or memory safety.

## Joint treatment and amortization

The experiment observes the result of compiler CGU16 plus ThinLTO, including
preparation/native build effects, against CGU256 without LTO. It does not observe
CGU16 alone or ThinLTO alone. Infinitely many allocations of individual effects
and their interaction yield the same joint result. Individual causal effects
are therefore unidentified without additional treatments and assumptions.

If a future matched cost study establishes extra compiler-construction cost `D`
and per-invocation saving `s>0`, reuse over `N` invocations saves total time exactly
when `N*s>D` in the additive model. For `D>=0`, the least integer satisfying this
strict inequality is `floor(D/s)+1`. No matched construction-cost estimate was
made here, so this formula supplies no numerical break-even count.

The argument-preservation and resource proofs elsewhere remain conditional on
their stated Cargo, compiler, OS and process contracts. Hash bindings assume
collision resistance and preserved execution records. Finite tests and hashes
are not a formal proof of compiler semantics, historical attestation or absence
of unsafe behavior in dependencies.
