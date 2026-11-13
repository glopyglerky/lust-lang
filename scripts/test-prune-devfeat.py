#!/usr/bin/env python3
# Unit tests for dev feature selection in bench/slices/prune.py
# (docs/dev-features.md): config from the ts-rust tree, flag and env
# overrides, and the lust/rustc flag forms.
# Usage: python3 scripts/test-prune-devfeat.py   (exit 0 = pass)
import importlib.util
import os
import tempfile
from pathlib import Path

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("prune", os.path.join(HERE, "..", "bench", "slices", "prune.py"))
prune = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prune)

failures = []


def check(name, cond, detail=""):
    print(f"{'ok' if cond else 'FAIL'} {name}" + (f": {detail}" if detail and not cond else ""))
    if not cond:
        failures.append(name)


with tempfile.TemporaryDirectory() as td:
    root = Path(td)
    pkg = root / "crates" / "ts_goport"
    (pkg / "src").mkdir(parents=True)
    (pkg / "Cargo.toml").write_text(
        '[package]\nname = "ts_goport"\nversion = "0.0.0"\nedition = "2024"\n\n'
        '[package.metadata.lust]\ndev-exclude-features = ["jemalloc"]\n\n'
        '[features]\ndefault = ["jemalloc", "fast"]\njemalloc = ["dep:tikv-jemalloc-ctl"]\nfast = []\n'
    )
    spec_json = {"tsrust_dir": str(root), "src_dir": "crates/ts_goport/src"}
    feats, where = prune.dev_features_for(spec_json, env={})
    check("package config excludes jemalloc", feats == ["fast"], f"{feats} ({where})")
    feats, _ = prune.dev_features_for(spec_json, cli_features="jemalloc", env={})
    check("flag exact list wins", feats == ["jemalloc"], str(feats))
    feats, _ = prune.dev_features_for(spec_json, env={"LUST_DEV_EXCLUDE_FEATURES": "fast,jemalloc"})
    check("env exclude wins over config", feats == [], str(feats))
    # .lust.toml at the workspace root when the manifest has no table.
    (pkg / "Cargo.toml").write_text(
        '[package]\nname = "ts_goport"\nversion = "0.0.0"\nedition = "2024"\n\n'
        '[features]\ndefault = ["jemalloc"]\njemalloc = []\nnoembed = []\n'
    )
    (root / ".lust.toml").write_text('dev-features = ["noembed"]\n')
    feats, where = prune.dev_features_for(spec_json, env={})
    check("dotfile exact list", feats == ["noembed"], f"{feats} ({where})")
    (root / ".lust.toml").unlink()
    feats, where = prune.dev_features_for(spec_json, env={})
    check("no config is old behavior", feats == [] and "no lust dev config" in where, f"{feats} ({where})")

prune.DEV_FEATURES = ["fast"]
check("lust flag form", prune.lust_feature_args() == ["--features", "fast"])
check("rustc flag form", prune.rustc_feature_args() == ["--cfg", 'feature="fast"'])
prune.DEV_FEATURES = []
check("empty list is no flags", prune.lust_feature_args() == [] and prune.rustc_feature_args() == [])

print(f"test-prune-devfeat: {'FAILED ' + repr(failures) if failures else 'all passed'}")
raise SystemExit(1 if failures else 0)
