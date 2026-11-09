# Conditional analysis: definitions and proofs

This document proves properties of the estimator, finite randomization test and
idealized scheduling bounds. It does not prove compiler semantics, workload
equivalence, empirical confidence coverage, or a measured performance gain.

Let `(B_i,C_i)`, `i=1,...,n`, be paired positive finite wall times for equivalent
work. Define `d_i = ln(B_i) - ln(C_i)` and
`G = exp((1/n) sum_i d_i)`. Implementation uses the difference of logarithms to
avoid overflow in the intermediate division. It rejects zero, negative,
nonfinite and nonrepresentable output ratios rather than silently clipping them.
The mathematical claims below use real arithmetic; floating-point results have
rounding error. No absolute numerical exactness is claimed.

## Estimator invariants

By logarithm/exponential identities,

```text
G = (product_i (B_i / C_i))^(1/n).
```

Thus `G` is the geometric mean of paired ratios, not a ratio of arithmetic means.
Since each `d_i` lies between their minimum and maximum, their arithmetic mean
does too. Exponentiation is monotone, so
`min_i(B_i/C_i) <= G <= max_i(B_i/C_i)`. Multiplying both times in any pair by
any positive common factor leaves that pair's `d_i` unchanged. Reversing all
lanes negates every `d_i` and replaces `G` by `1/G`. If all pairs are equal,
`d_i=0` and `G=1`; if all ratios are `r`, `G=r`. These are algebraic invariants,
not evidence that experiment inputs were equivalent.

The implementation reports conventional medians (average of two middle values
for even `n`) and nearest-rank p95 at sorted index `ceil(.95*n)-1`, separately
for each lane. It retains raw ratios and raw paired order. Tail estimates from
eight observations are imprecise; p95 at that size is simply the maximum.

## Bootstrap construction and limits

For each resample, draw `n` indices independently with replacement from the
observed pairs, preserving the baseline/candidate pairing. Compute `G*` by the
same log estimator. Sort `R` resampled estimates and take indices
`floor(.025*(R-1))` and `ceil(.975*(R-1))` as the reported percentile endpoints.
This defines the algorithm precisely, including discrete quantiles. It is not a
proof that the interval has 95% coverage for real build workloads.

Each resampled log mean is a convex combination of the observed logs, so each
resampled estimate and hence both endpoints lie within the observed min/max
ratios in real arithmetic. Reversing lanes with the same sequence of sampled
indices reciprocates every bootstrap estimate; sorting reverses the sequence.
The selected endpoint indices sum to `R-1`, so the reversed interval is
`[1/upper, 1/lower]` in real arithmetic. Numerical tests allow rounding tolerance.

SplitMix64 version 1 with the explicit seed makes the resampling sequence
reproducible. Index selection rejects values below `2^64 mod n` before taking
modulo `n`. On a uniform 64-bit draw, the accepted range has a size divisible by
`n`, and each remainder has equal preimage count. This proves absence of modulo
bias conditional on uniform draws. SplitMix64 is a deterministic pseudorandom
generator; the seed is not a proof of random independent host measurements.

Interpreting this bootstrap interval requires independent representative pairs
and equivalent workloads. Sequential builds share disk caches, thermal state,
background load and scheduling history; these can violate independence.
Balanced order mitigates some order bias without proving its absence. Do not
convert the interval into a compiler-correctness or generalization guarantee.

## Exact finite sign-flip test

For `n <= 16`, enumerate every `s in {-1,+1}^n` and compute
`T(s) = abs((1/n) sum_i s_i*d_i)`. The observed statistic is
`T_obs = abs(mean(d))`. Count assignments with `T(s) >= T_obs`, including ties,
and divide by `2^n`. Zero log differences remain in the enumeration: their two
signs produce duplicate values, correctly preserving multiplicity. All-zero
observations have p-value one. Beyond 16 pairs this runner reports null instead
of calling a simulation exact.

Assume under the null that conditional on the sign-flip orbit, the observed sign
assignment is uniform on those `2^n` assignments (sign exchangeability). Counting
extreme assignments gives the exact conditional tail probability under that
assumption. To see validity with ties, sort all assignments by statistic. For any
threshold `alpha`, assignments with tail proportion at most `alpha` occupy at
most `alpha * 2^n` slots; a tie group is included as a whole. Thus the conditional
probability of `p <= alpha` is at most `alpha`. This is a finite randomization
property, not proof that measured build timings satisfy exchangeability.
Independence alone does not imply sign symmetry; the assumption is substantive.

The implementation uses a symmetric floating tolerance of
`64*epsilon*max(1,sum(abs(d_i)))` in tail comparisons, conservatively including
near ties. “Exact” refers to enumerating all sign assignments, not exact-real
arithmetic. Reversing lanes leaves all absolute sign statistics unchanged.
For eight strictly positive identical logs only the all-positive and all-negative
assignments attain the maximum, giving `2/256`. For two equal positive logs and
one zero log, four of eight assignments attain the maximum, giving `1/2`.

Testing many projects, profiles, worker counts and compiler versions introduces
multiple comparisons and selection effects. These intervals and p-values are
exploratory and unadjusted. Select primary comparisons/sample counts before
viewing results; retain regressions, failures and all observations. Statistical
significance does not resolve confounding by different toolchains or work.

## Idealized work/span and Amdahl bounds

For a computation DAG with total work `W`, critical path `S`, and `P` identical
processors, any legal schedule has `T_P >= W/P` because at most `P` units of work
can be done per unit time. Also `T_P >= S` because precedence forces every
critical-path operation to run in order. Therefore
`T_P >= max(W/P,S)`. If serial execution takes exactly `W`, its ideal speedup is
at most `min(P,W/S)`. Communication, scheduling, contention and extra work can
only invalidate an assumption or increase the actual time; they do not establish
that the bound is attainable.

If a baseline time `T` consists of unchanged fraction `1-p` and accelerable
fraction `p`, and the latter improves by factor `s>0` without other changes,
then `T_new = T*((1-p)+p/s)` and speedup is
`1/((1-p)+p/s)`. For `0<=p<1`, as `s` tends to infinity, this is bounded by
`1/(1-p)`. This follows directly by partitioning and summing the times. With
nonnegative coordination overhead `H`, add `H` to `T_new`; speedup can decrease.
Measured compiler stages can overlap and share resources, so the additive model
requires evidence before application. None of these formulas infer an unmeasured
Lustr/BlueFe gain, identify the true fraction `p`, or prove full Rust support.
