#!/usr/bin/env python3
"""Regression tests for fail-closed compiler acceptance evidence."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("native_speed", ROOT / "tools/qualify-native-speed.py")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def observations(candidate=80):
    return [{"sample": sample, "baseline_ns": 100, "candidate_ns": candidate,
             "correct": True, "source_equal": True} for sample in range(12)]


def fresh_cache_observations():
    rows = observations()
    for row in rows:
        row['cold'] = True
        for lane in ('baseline', 'candidate'):
            runtime = f"/owned/{lane}/runtime/{row['sample']}"
            cache = f"/owned/{lane}/compile-cache/{row['sample']}"
            row[lane + '_runtime_temp'] = {'fresh': True, 'empty_before_compile': True, 'path': runtime}
            row[lane + '_tmpdir'] = runtime
            row[lane + '_compile_cache'] = {'fresh': True, 'empty_before_compile': True,
                                            'enabled': True, 'path': cache}
            row[lane + '_cache_env'] = {'LUST_CACHE_DIR': cache, 'LUST_NO_CACHE': None,
                                       'LUST_NO_DAEMON': '1'}
    return rows


def complete_report():
    names = ("identities", "provenance", "input-stability", "upstream-preservation", "upstream-accepted-corpus", "engine-trace", "concurrent-cargo", "mixed-compatibility", "full-rust-compatibility", "build-scripts", "doctests")
    return {"requirements": {name: {"state": "passed"} for name in names},
            "series": {name: {"state": "passed"} for name in MODULE.expected_series()},
            "failures": [], "diagnostic_without_trace": False, "functional_only": False}


def trace(compiler="/compiler/lustr-native", second_start=2.1):
    return (
        '1 1.000000 execve("/tools/cargo", ["cargo", "build"], 0x0) = 0\n'
        f'2 2.000000 execve("{compiler}", ["{compiler}", "--crate-name", "left", "left/src/lib.rs"], 0x0) = 0\n'
        f'3 {second_start:.6f} execve("{compiler}", ["{compiler}", "--crate-name", "right", "right/src/lib.rs"], 0x0) = 0\n'
        '2 3.000000 exit_group(0) = ?\n'
        '3 4.000000 exit_group(0) = ?\n'
        '1 5.000000 exit_group(0) = ?\n'
    )


def parse(text, overlap=True):
    return MODULE.parse_trace(text, Path("/tools/cargo"), Path("/compiler/lustr-native"), Path("/oracle/rustc"), True, overlap)


class Statistics(unittest.TestCase):
    def test_faster_complete_rows_pass_with_zero_margin(self):
        result = MODULE.paired_decision(observations())
        self.assertEqual(result["state"], "passed")
        self.assertEqual(result["regression_budget"], 0)
        self.assertLess(result["upper_95"], 1)

    def test_single_slower_observation_is_not_averaged_away(self):
        rows = observations(10)
        rows[5]["candidate_ns"] = 101
        result = MODULE.paired_decision(rows)
        self.assertLess(result["geometric_ratio"], 1)
        self.assertEqual(result["state"], "failed")

    def test_exact_equality_satisfies_at_least_as_fast(self):
        result = MODULE.paired_decision(observations(100))
        self.assertEqual(result["state"], "passed")
        self.assertEqual(result["upper_95"], 1.0)

    def test_noise_crossing_threshold_cannot_qualify(self):
        rows = observations(99)
        for row in rows[::2]:
            row["candidate_ns"] = 101
        self.assertFalse(MODULE.paired_decision(rows)["qualified"])

    def test_missing_and_duplicate_pairs_do_not_pass(self):
        for rows in (observations()[:-1], observations() + [observations()[0]]):
            self.assertFalse(MODULE.paired_decision(rows)["qualified"])

    def test_invalid_measurements_or_wrong_sources_do_not_pass(self):
        for field, value in (("candidate_ns", 0), ("candidate_ns", -1), ("candidate_ns", float("nan")),
                             ("candidate_ns", True), ("source_equal", False), ("correct", False)):
            rows = observations()
            rows[2][field] = value
            self.assertFalse(MODULE.paired_decision(rows)["qualified"], (field, value))

    def test_bootstrap_is_reproducible(self):
        rows = observations()
        for number, row in enumerate(rows):
            row["candidate_ns"] += number
        self.assertEqual(MODULE.paired_decision(rows), MODULE.paired_decision(rows))

    def test_cold_rows_require_new_empty_per_lane_runtime_cache(self):
        rows = observations()
        for row in rows:
            row['cold'] = True
        self.assertFalse(MODULE.paired_decision(rows)['qualified'])
        for row in rows:
            for lane in ('baseline', 'candidate'):
                row[lane + '_runtime_temp'] = {'fresh': True, 'empty_before_compile': True,
                                               'path': f"/owned/{lane}/{row['sample']}"}
                row[lane + '_tmpdir'] = row[lane + '_runtime_temp']['path']
        self.assertTrue(MODULE.paired_decision(rows)['qualified'])
        wrong_config = copy.deepcopy(rows)
        wrong_config[0]['candidate_tmpdir'] = '/tmp'
        self.assertFalse(MODULE.paired_decision(wrong_config)['qualified'])
        for field, value in (('fresh', False), ('empty_before_compile', False), ('path', '/owned/baseline/0')):
            invalid = copy.deepcopy(rows)
            invalid[1]['candidate_runtime_temp'][field] = value
            self.assertFalse(MODULE.paired_decision(invalid)['qualified'])

    def test_cold_rows_with_absent_nested_evidence_are_unproved(self):
        rows = observations()
        rows[0].update(cold=True, baseline_runtime_temp=None, candidate_runtime_temp=None)
        self.assertEqual(MODULE.paired_decision(rows)['state'], 'unproved')

    def test_fresh_cache_rows_require_enabled_empty_distinct_caches(self):
        self.assertTrue(MODULE.paired_decision(fresh_cache_observations(), fresh_cache=True)['qualified'])
        self.assertFalse(MODULE.paired_decision(observations(), fresh_cache=True)['qualified'])
        for lane in ('baseline', 'candidate'):
            for field, value in (('fresh', False), ('empty_before_compile', False), ('enabled', False),
                                 ('path', f'/owned/{lane}/compile-cache/0')):
                rows = fresh_cache_observations()
                rows[1][lane + '_compile_cache'][field] = value
                self.assertFalse(MODULE.paired_decision(rows, fresh_cache=True)['qualified'], (lane, field))
            rows = fresh_cache_observations()
            rows[0][lane + '_compile_cache'] = None
            self.assertFalse(MODULE.paired_decision(rows, fresh_cache=True)['qualified'])
        rows = fresh_cache_observations()
        rows[0]['candidate_compile_cache']['path'] = rows[0]['baseline_compile_cache']['path']
        rows[0]['candidate_cache_env']['LUST_CACHE_DIR'] = rows[0]['baseline_compile_cache']['path']
        self.assertFalse(MODULE.paired_decision(rows, fresh_cache=True)['qualified'])

    def test_fresh_cache_evidence_must_match_actual_compiler_environment(self):
        for lane in ('baseline', 'candidate'):
            for field, value in (('LUST_CACHE_DIR', '/different/cache'), ('LUST_NO_CACHE', '1'),
                                 ('LUST_NO_DAEMON', None), ('LUST_NO_DAEMON', '0')):
                rows = fresh_cache_observations()
                rows[1][lane + '_cache_env'][field] = value
                self.assertFalse(MODULE.paired_decision(rows, fresh_cache=True)['qualified'], (lane, field, value))
            rows = fresh_cache_observations()
            del rows[1][lane + '_cache_env']['LUST_NO_CACHE']
            self.assertFalse(MODULE.paired_decision(rows, fresh_cache=True)['qualified'])


class Decisions(unittest.TestCase):
    def test_fresh_cache_compile_and_run_are_required_series(self):
        self.assertEqual(len(MODULE.expected_series()), 53)
        self.assertEqual(MODULE.DIRECT_MODES.count('fresh-cache'), 1)
        self.assertIn('lust/fresh-cache', MODULE.expected_series())
        self.assertIn('lust/fresh-cache/compile-plus-run', MODULE.expected_series())
        self.assertEqual(set(MODULE.COLD_DIRECT_MODES), {'process-cold', 'fresh-cache', 'cold-invalid'})

    def test_complete_finite_report_is_not_a_universal_claim(self):
        result = MODULE.final_decision(complete_report())
        self.assertTrue(result["qualified"])
        self.assertFalse(result["universal_speed_proved"])
        self.assertFalse(result["universal_coverage_proved"])

    def test_every_missing_required_mode_blocks(self):
        for name in MODULE.expected_series():
            report = complete_report()
            del report["series"][name]
            result = MODULE.final_decision(report)
            self.assertFalse(result["qualified"], name)
            self.assertIn(name, result["blocking"])

    def test_every_missing_requirement_blocks(self):
        for name in complete_report()["requirements"]:
            report = complete_report()
            del report["requirements"][name]
            self.assertFalse(MODULE.final_decision(report)["qualified"], name)

    def test_diagnostic_and_retained_failure_never_qualify(self):
        for key, value in (("diagnostic_without_trace", True), ("functional_only", True), ("failures", [{"scope": "native compile", "error": "failed"}])):
            report = complete_report()
            report[key] = value
            self.assertFalse(MODULE.final_decision(report)["qualified"])


class ProcessEvidence(unittest.TestCase):
    def test_real_source_process_intervals_prove_observed_overlap(self):
        result = parse(trace())
        self.assertEqual(result["overlapping_compiler_pids"], [(2, 3)])
        self.assertEqual(len(result["compilations"]), 2)

    def test_sequential_compilation_is_not_parallel(self):
        with self.assertRaisesRegex(ValueError, "no overlapping"):
            parse(trace(second_start=3.1))

    def test_stock_fallback_cannot_be_hidden_among_native_processes(self):
        rogue = '4 2.500000 execve("/oracle/rustc", ["/oracle/rustc", "--crate-name", "hidden", "hidden.rs"], 0x0) = 0\n'
        with self.assertRaisesRegex(ValueError, "unexpected compiler"):
            parse(trace() + rogue)

    def test_compiler_name_at_other_path_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "unexpected compiler"):
            parse(trace().replace('"/compiler/lustr-native"', '"/another/lustr-native"'))

    def test_version_queries_do_not_count_as_source_compilation(self):
        text = trace().replace('"--crate-name", "left", "left/src/lib.rs"', '"-vV"').replace('"--crate-name", "right", "right/src/lib.rs"', '"--print", "cfg"')
        with self.assertRaisesRegex(ValueError, "no actual source"):
            parse(text)

    def test_missing_exit_or_nonzero_exit_blocks(self):
        for text in (trace().replace('2 3.000000 exit_group(0) = ?\n', ''), trace().replace('2 3.000000 exit_group(0)', '2 3.000000 exit_group(1)')):
            with self.assertRaises(ValueError):
                parse(text)

    def test_missing_cargo_or_relative_exec_blocks(self):
        for text in ('\n'.join(trace().splitlines()[1:]), trace().replace('"/tools/cargo"', '"cargo"')):
            with self.assertRaises(ValueError):
                parse(text)

    def test_unfinished_exec_is_joined_and_incomplete_one_rejected(self):
        original = '2 2.000000 execve("/compiler/lustr-native", ["/compiler/lustr-native", "--crate-name", "left", "left/src/lib.rs"], 0x0) = 0'
        replacement = original.removesuffix(' = 0') + ' <unfinished ...>\n2 2.010000 <... execve resumed>) = 0'
        self.assertEqual(parse(trace().replace(original, replacement))["state"], "passed")
        with self.assertRaisesRegex(ValueError, "incomplete"):
            parse(trace().replace(original, replacement.splitlines()[0]))

    def test_execveat_without_resolved_identity_is_unqualified(self):
        with self.assertRaisesRegex(ValueError, "execveat"):
            parse(trace() + '4 2.500000 execveat(3, "", ["compiler"], 0x0, AT_EMPTY_PATH) = 0\n')


class FailureRetention(unittest.TestCase):
    def args(self, output):
        return argparse.Namespace(repository=ROOT, output=output, native=Path('/bin/true'), lust=Path('/bin/true'),
                                  cargo=Path('/bin/true'), rustc=Path('/bin/true'), samples=12, timeout=1,
                                  diagnostic_without_trace=True, functional_only=True,
                                  native_build_receipt=None, tcc_root=None)

    def test_same_fake_compiler_for_every_role_cannot_qualify(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'evidence'
            campaign = MODULE.Campaign(self.args(output))
            report = campaign.run()
            self.assertFalse(report['qualified'])
            self.assertEqual(report['requirements']['identities']['state'], 'failed')
            retained = json.loads((output / 'report.json').read_text())
            self.assertTrue(retained['failures'])
            self.assertEqual(set(retained['series']), set(MODULE.expected_series()))

    def test_real_process_failure_keeps_raw_streams_before_reporting_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            campaign = MODULE.Campaign(self.args(Path(temporary) / 'evidence'))
            env = campaign.env('candidate', campaign.work / 'environment')
            row, stdout, stderr = campaign.execute('failed-command', ['/bin/sh', '-c', 'printf expected; printf diagnostic >&2; exit 7'], campaign.work, env)
            self.assertEqual(row['exit_code'], 7)
            self.assertEqual((campaign.output / row['stdout_file']).read_bytes(), b'expected')
            self.assertEqual((campaign.output / row['stderr_file']).read_bytes(), b'diagnostic')

    def test_old_evidence_is_not_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(ValueError, 'output already exists'):
                MODULE.Campaign(self.args(Path(temporary)))


class RuntimeCacheLifecycle(unittest.TestCase):
    def test_fresh_direct_samples_keep_caching_enabled_and_reset_both_caches(self):
        with tempfile.TemporaryDirectory() as temporary:
            args = FailureRetention().args(Path(temporary) / 'evidence')
            args.functional_only = False
            args.samples = 12
            campaign = MODULE.Campaign(args)
            family = campaign.work / 'direct'
            family.mkdir()
            visited, commands = set(), []

            def execute(label, argv, cwd, env, trace=False):
                cache_env = {name: env.get(name) for name in ('LUST_CACHE_DIR', 'LUST_NO_CACHE', 'LUST_NO_DAEMON')}
                row = {'elapsed_ns': 100, 'exit_code': 0, 'compiler_tmpdir': env['TMPDIR'], 'compiler_cache': cache_env}
                if not label.endswith('-run'):
                    self.assertIsNone(env.get('LUST_NO_CACHE'))
                    self.assertEqual(env['LUST_NO_DAEMON'], '1')
                    for key in ('TMPDIR', 'LUST_CACHE_DIR'):
                        directory = Path(env[key])
                        self.assertNotIn(directory, visited)
                        self.assertEqual(list(directory.iterdir()), [])
                        visited.add(directory)
                        (directory / 'retained-cache-object').write_bytes(b'previous compilation')
                    artifact = Path(argv[-1])
                    self.assertFalse(artifact.exists())
                    artifact.write_bytes(b'compiled output')
                    commands.append(label)
                return row, b'14\n', b''

            with mock.patch.object(campaign, 'execute', execute), mock.patch.object(MODULE.subprocess, 'Popen') as daemon:
                campaign.direct_mode(family, 'fn main() {}\n', 'fresh-cache')
                daemon.assert_not_called()
            self.assertEqual(len(commands), 26)
            self.assertEqual(len(visited), 52)
            for name in ('lust/fresh-cache', 'lust/fresh-cache/compile-plus-run'):
                self.assertTrue(campaign.report['series'][name]['qualified'])
                self.assertEqual(len(campaign.report['series'][name]['observations']), 12)

    def test_cold_dirs_start_empty_and_warm_dirs_retain_runtime_objects(self):
        helper = MODULE.load_helper(ROOT, 'qualify-native-cargo.py')
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            cold = helper.prepare_runtime_temp(root, 'cold-first', True)
            self.assertTrue(cold['fresh'] and cold['empty_before_compile'])
            cache = Path(cold['path']) / 'lust-runtime-example.o'
            cache.write_bytes(b'observed runtime object')
            warm = helper.prepare_runtime_temp(root, 'cold-first', False)
            self.assertFalse(warm['fresh'])
            self.assertFalse(warm['empty_before_compile'])
            self.assertEqual(warm['retained_entries_before_compile'], ['lust-runtime-example.o'])
            fresh = helper.prepare_runtime_temp(root, 'cold-second', True)
            self.assertTrue(fresh['empty_before_compile'])
            self.assertNotEqual(fresh['path'], warm['path'])
            with self.assertRaises(FileExistsError):
                helper.prepare_runtime_temp(root, 'cold-first', True)
            self.assertEqual(cache.read_bytes(), b'observed runtime object')

    def test_unsafe_temp_paths_are_rejected_without_mutation(self):
        helper = MODULE.load_helper(ROOT, 'qualify-native-cargo.py')
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for label in ('../outside', '', 'a/b', '/tmp'):
                with self.assertRaises(ValueError):
                    helper.prepare_runtime_temp(root, label, True)
            (root / 'runtime-temporaries').symlink_to('/tmp', target_is_directory=True)
            with self.assertRaises(ValueError):
                helper.prepare_runtime_temp(root, 'cold', True)


class EditRebuildEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.helper = MODULE.load_helper(ROOT, 'qualify-native-cargo.py')

    def evidence(self, fresh=False):
        return ([{'reason': 'compiler-artifact', 'target': {'name': 'example', 'kind': ['bin']},
                  'fresh': fresh}],
                [{'argv': ['/compiler/lustr-native', '--crate-name', 'example', 'src/main.rs'],
                  'environment': {}}])

    def check(self, events, calls, fresh=False):
        return self.helper.compiler_rebuild_evidence(
            events, calls, Path('/compiler/lustr-native'), Path('/owned/project'),
            Path('/owned/project/src/main.rs'), 'example', fresh)

    def test_all_cargo_families_collect_type_and_macro_modes_once(self):
        for family, modes in (('cargo', MODULE.CARGO_MODES), ('mixed', MODULE.MIXED_MODES),
                              ('full-rust', MODULE.FULL_MODES)):
            for mode in ('type-edit', 'macro-edit'):
                self.assertEqual(modes.count(mode), 1, (family, mode))
                self.assertEqual(MODULE.expected_series().count(f'{family}/{mode}'), 1)

    def test_changed_input_requires_real_source_compilation(self):
        events, calls = self.evidence()
        result = self.check(events, calls)
        self.assertEqual(result['compiler_invocations'], calls)
        for invalid in ([], [{'argv': ['/compiler/lustr-native', '-vV']}],
                        [{'argv': ['/other/lustr-native', '--crate-name', 'example', 'src/main.rs']}],
                        [{'argv': ['/compiler/lustr-native', '--crate-name', 'example', 'src/other.rs']}],
                        [{'argv': ['/compiler/lustr-native', '--crate-name', 'other', 'src/main.rs']}]):
            with self.subTest(calls=invalid), self.assertRaises(ValueError):
                self.check(events, invalid)

    def test_fresh_artifact_cannot_count_as_an_edit(self):
        events, calls = self.evidence(fresh=True)
        with self.assertRaises(ValueError):
            self.check(events, calls)

    def test_noop_requires_no_source_compilation(self):
        events, calls = self.evidence(fresh=True)
        self.assertEqual(self.check(events, [], fresh=True)['compiler_invocations'], [])
        with self.assertRaises(ValueError):
            self.check(events, calls, fresh=True)

    def test_missing_duplicate_or_nonbinary_artifacts_cannot_prove_rebuild(self):
        events, calls = self.evidence()
        for invalid in ([], events + events,
                        [{'reason': 'compiler-artifact', 'target': {'name': 'example', 'kind': ['lib']}, 'fresh': False}]):
            with self.subTest(events=invalid), self.assertRaises(ValueError):
                self.check(invalid, calls)


if __name__ == '__main__':
    unittest.main()
