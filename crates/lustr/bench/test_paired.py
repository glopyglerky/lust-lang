import unittest

from paired import paired_summary


class PairedTests(unittest.TestCase):
    def test_identical_observations_have_no_gain_or_evidence(self):
        result = paired_summary([(10, 10)] * 8, 100)
        self.assertEqual(result["baseline_over_candidate_geomean"], 1)
        self.assertEqual(result["geomean_ratio_bootstrap_95_percent"], [1, 1])
        self.assertEqual(result["two_sided_sign_flip_p"], 1)

    def test_known_gain_and_reverse_are_reciprocals(self):
        forward = paired_summary([(20, 10)] * 8, 100)
        reverse = paired_summary([(10, 20)] * 8, 100)
        self.assertAlmostEqual(forward["baseline_over_candidate_geomean"], 2)
        self.assertAlmostEqual(reverse["baseline_over_candidate_geomean"], .5)
        self.assertEqual(forward["two_sided_sign_flip_p"], 2 / 256)
        self.assertEqual(forward["two_sided_sign_flip_p"], reverse["two_sided_sign_flip_p"])

    def test_bad_measurements_cannot_become_ratios(self):
        for pairs in ([], [(0, 1)], [(1, float("inf"))], [(1, float("nan"))]):
            with self.assertRaises(ValueError):
                paired_summary(pairs)


if __name__ == "__main__":
    unittest.main()
