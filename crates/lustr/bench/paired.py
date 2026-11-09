"""Paired timing estimates, conditional on valid equivalent-work observations.

Intervals describe these samples, not compiler correctness or other workloads.
The sign-flip test assumes exchangeable paired log differences under the null.
Sequential host measurements can violate that assumption. Keep raw order data.
"""
import itertools
import math
import random
import statistics


def paired_summary(pairs, resamples=10000):
    pairs = list(pairs)
    if not pairs or resamples < 1 or any(not math.isfinite(v) or v <= 0 for pair in pairs for v in pair) or any(len(pair) != 2 for pair in pairs):
        raise ValueError("positive finite baseline/candidate pairs required")
    logs = [math.log(a / b) for a, b in pairs]
    rng = random.Random(0x1A57)
    boot = sorted(math.exp(statistics.mean(rng.choices(logs, k=len(logs)))) for _ in range(resamples))
    observed = abs(statistics.mean(logs))
    # Exact enumeration for bounded studies; deterministic Monte Carlo beyond 16.
    exact = len(logs) <= 16
    signs = itertools.product((-1, 1), repeat=len(logs)) if exact else (rng.choices((-1, 1), k=len(logs)) for _ in range(resamples))
    extreme = total = 0
    for sign in signs:
        value = abs(statistics.mean(x * s for x, s in zip(logs, sign)))
        extreme += value >= observed - 1e-14
        total += 1
    return {
        "pairs": len(pairs), "baseline_over_candidate_geomean": math.exp(statistics.mean(logs)),
        "geomean_ratio_bootstrap_95_percent": [boot[math.floor(.025 * (resamples - 1))], boot[math.ceil(.975 * (resamples - 1))]],
        "two_sided_sign_flip_p": extreme / total if exact else (extreme + 1) / (total + 1),
        "sign_flip_exact": exact, "bootstrap_resamples": resamples, "seed": 0x1A57,
        "assumptions": "equivalent work; exchangeable paired log differences; independent pairs for bootstrap; conditional exploratory inference, no multiple-comparison correction",
    }
