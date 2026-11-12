#!/usr/bin/env python3
"""Check that missing, slower and inconclusive native samples cannot pass."""
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location("native_speed", Path(__file__).with_name("compare-native-speed.py"))
speed = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(speed)


class ComparisonTests(unittest.TestCase):
    def rows(self, ratios):
        return [{"upstream_ns": 10000, "candidate_ns": 10000 * ratio} for ratio in ratios]

    def test_missing_or_unbalanced_samples_do_not_pass(self):
        for count in (0, 1, 10, 11, 13):
            self.assertEqual(speed.decide(self.rows([0.9] * count))["state"], "unproved")

    def test_faster_samples_pass_without_a_regression_allowance(self):
        result = speed.decide(self.rows([0.9] * 12))
        self.assertEqual(result["state"], "passed")
        self.assertEqual(result["regression_allowance"], 0.0)
        self.assertEqual(result["threshold"], 1.0)

    def test_slower_samples_fail_even_for_a_small_regression(self):
        result = speed.decide(self.rows([1.001] * 12))
        self.assertEqual(result["state"], "regressed")

    def test_noise_does_not_prove_no_regression(self):
        result = speed.decide(self.rows([0.9, 1.1] * 6))
        self.assertNotEqual(result["state"], "passed")

    def test_faster_median_cannot_hide_a_slower_tail(self):
        result = speed.decide(self.rows([0.8] * 22 + [1.3] * 2))
        self.assertLess(result["median_ratio"], 1.0)
        self.assertGreater(result["p95_ratio"], 1.0)
        self.assertNotEqual(result["state"], "passed")


if __name__ == "__main__":
    unittest.main()
