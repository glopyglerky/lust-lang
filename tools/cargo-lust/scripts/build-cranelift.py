#!/usr/bin/env python3
"""Build the pinned Linux x86-64 Cranelift complement with real panic unwinding."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
PIN = ROOT / "backends" / "cranelift.json"
LIBRARY = "librustc_codegen_cranelift.so"
PROBE = r'''
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
struct DropProbe(Arc<AtomicUsize>);
impl Drop for DropProbe {
    fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); }
}
fn main() {
    assert!(cfg!(panic = "unwind"));
    let count = Arc::new(AtomicUsize::new(0));
    let captured = count.clone();
    let caught = std::panic::catch_unwind(move || {
        let _guard = DropProbe(captured);
        panic!("expected backend qualification panic");
    });
    assert!(caught.is_err(), "the backend did not catch the panic");
    assert_eq!(count.load(Ordering::SeqCst), 1, "the backend skipped cleanup");
    println!("catch_unwind=1 drops=1");
}
'''


class SetupError(Exception):
    pass


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def run(command: list[str], cwd: Path, env: dict[str, str], capture: bool = False) -> str:
    print("+ " + shlex.join(command), file=sys.stderr, flush=True)
    result = subprocess.run(command, cwd=cwd, env=env, check=True, text=True,
                            stdout=subprocess.PIPE if capture else sys.stderr)
    return result.stdout.strip() if capture else ""


def atomic_text(path: Path, text: str) -> None:
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        stream.write(text)
        stream.flush()
        os.fsync(stream.fileno())
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def owned_input(path: Path, text: str) -> None:
    if path.exists():
        if path.read_bytes() != text.encode("utf-8"):
            raise SetupError(f"Owned bootstrap input changed: {path.name}; review it before rerunning setup.")
    else:
        atomic_text(path, text)


def verify_source(git: str, source: Path, pin: dict, env: dict[str, str]) -> None:
    def query(*args: str) -> str:
        return run([git, "-C", str(source), *args], source, env, capture=True)
    if query("rev-parse", "HEAD") != pin["revision"]:
        raise SetupError("Existing backend source is at a different revision; use a new output directory.")
    if query("remote", "get-url", "origin") != pin["repository"]:
        raise SetupError("Existing backend source has a different origin.")
    if query("status", "--porcelain=v1", "--untracked-files=all"):
        raise SetupError("Backend source has local changes; review them before rerunning setup.")
    if query("rev-parse", "HEAD:Cargo.lock") != pin["cargo_lock_git_blob"]:
        raise SetupError("The pinned Cargo.lock identity does not match the checked-out source.")


def publish_library(source: Path, output: Path) -> tuple[Path, str]:
    digest = sha256(source)
    directory = output / "lib" / digest
    directory.mkdir(parents=True, exist_ok=True)
    destination = directory / LIBRARY
    if destination.exists() or destination.is_symlink():
        if destination.is_symlink() or not destination.is_file() or sha256(destination) != digest:
            raise SetupError("An existing content-addressed backend has different content; refusing to replace it.")
        return destination, digest
    with tempfile.NamedTemporaryFile(dir=directory, delete=False) as stream:
        temporary = Path(stream.name)
        with source.open("rb") as original:
            shutil.copyfileobj(original, stream)
        stream.flush()
        os.fsync(stream.fileno())
    try:
        if sha256(temporary) != digest:
            raise SetupError("Backend changed while it was being published.")
        os.chmod(temporary, 0o644)
        # link is atomic and refuses to overwrite an existing destination.
        os.link(temporary, destination)
    finally:
        temporary.unlink(missing_ok=True)
    return destination, digest


def setup(args: argparse.Namespace) -> None:
    if platform.system() != "Linux" or platform.machine() != "x86_64":
        raise SetupError("This backend build is qualified only for Linux x86_64.")
    import fcntl

    pin = json.loads(PIN.read_text())
    if (pin.get("schema_version") != 1 or pin.get("host") != "x86_64-unknown-linux-gnu"
            or pin.get("repository") != "https://github.com/rust-lang/rustc_codegen_cranelift.git"
            or not re.fullmatch(r"[0-9a-f]{40}", pin.get("revision", ""))
            or not re.fullmatch(r"nightly-\d{4}-\d{2}-\d{2}", pin.get("toolchain", ""))
            or not re.fullmatch(r"[0-9a-f]{40}", pin.get("cargo_lock_git_blob", ""))
            or pin.get("features") != ["unwinding"]):
        raise SetupError("Unrecognized backend pin; review the setup recipe before changing it.")
    tools = {name: shutil.which(name) for name in ("git", "rustup", "cc")}
    if any(value is None for value in tools.values()):
        raise SetupError("Setup requires git, rustup, and a working C linker named cc on PATH.")
    git, rustup = str(tools["git"]), str(tools["rustup"])
    output = Path(args.output_dir).absolute()
    output.mkdir(parents=True, exist_ok=True)
    owner = output / "owner.json"
    if owner.exists():
        if json.loads(owner.read_text()) != pin:
            raise SetupError("This output directory belongs to a different backend pin; select a new directory.")
    elif any(output.iterdir()):
        raise SetupError("Refusing to reuse a nonempty output directory without this setup's owner.json.")
    else:
        atomic_text(owner, json.dumps(pin, indent=2) + "\n")

    # Never rebuild or publish concurrently into the same owned directory.
    with (output / ".setup.lock").open("w") as lock:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise SetupError("Another backend setup is using this output directory.") from error

        env = os.environ.copy()
        for key in list(env):
            if key.startswith(("CARGO", "RUSTC", "RUSTDOC", "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES")) or key in ("RUSTFLAGS", "LUST_CARGO", "LUST_CRANELIFT_BACKEND"):
                del env[key]
        env.update({"CARGO_HOME": str(output / "cargo-home"), "CARGO_TERM_COLOR": "never", "RUSTUP_TOOLCHAIN": pin["toolchain"], "GIT_TERMINAL_PROMPT": "0"})
        source = output / "source"
        if not source.exists():
            with tempfile.TemporaryDirectory(prefix="source-download-", dir=output) as temporary:
                checkout = Path(temporary) / "source"
                run([git, "init", "--quiet", str(checkout)], output, env)
                run([git, "-C", str(checkout), "remote", "add", "origin", pin["repository"]], output, env)
                run([git, "-C", str(checkout), "fetch", "--depth", "1", "origin", pin["revision"]], output, env)
                run([git, "-C", str(checkout), "checkout", "--quiet", "--detach", pin["revision"]], output, env)
                verify_source(git, checkout, pin, env)
                os.rename(checkout, source)
        verify_source(git, source, pin, env)
        # The output may be inside cargo-lust's own workspace. A separate owned
        # bootstrap workspace prevents Cargo from inheriting that parent without
        # editing the pristine upstream manifest. Its lock is exactly upstream's.
        owned_input(output / "Cargo.toml", '[workspace]\nmembers = ["source"]\nresolver = "3"\n')
        owned_input(output / "Cargo.lock", (source / "Cargo.lock").read_bytes().decode("utf-8"))

        if not args.skip_install:
            run([rustup, "toolchain", "install", pin["toolchain"], "--profile", "minimal", "--component", "rustc-dev", "--component", "rust-src", "--component", "llvm-tools", "--no-self-update"], output, env)
        resolved = {}
        for name in ("cargo", "rustc", "rustdoc"):
            resolved[name] = run([rustup, "which", "--toolchain", pin["toolchain"], name], output, env, capture=True)
        rustc_version = run([resolved["rustc"], "-vV"], output, env, capture=True)
        if f"host: {pin['host']}" not in rustc_version.splitlines():
            raise SetupError("Installed compiler host does not match the qualified x86_64 GNU/Linux target.")
        cargo_version = run([resolved["cargo"], "--version", "--verbose"], output, env, capture=True)
        env.update({"CARGO": resolved["cargo"], "RUSTC": resolved["rustc"], "RUSTDOC": resolved["rustdoc"], "CARGO_BUILD_INCREMENTAL": "true", "RUSTFLAGS": "-Zallow-features=rustc_private,f16,f128"})
        command = [resolved["cargo"], "build", "--manifest-path", str(source / "Cargo.toml"), "--target-dir", str(output / "build"), "--target", pin["host"], "--locked", "--release", "--no-default-features", "--features", "unwinding", "-Zembed-metadata=no"]
        run(command, source, env)
        verify_source(git, source, pin, env)
        if sha256(output / "Cargo.lock") != sha256(source / "Cargo.lock"):
            raise SetupError("The bootstrap lockfile changed during the locked build.")
        built = output / "build" / pin["host"] / "release" / LIBRARY
        if not built.is_file():
            raise SetupError("Cargo did not produce the expected backend library.")
        # Qualify the exact immutable path consumers will load, including relocation.
        # On failure this may leave an unreferenced hash artifact, never an env export.
        library, digest = publish_library(built, output)

        probe_source = output / "unwind-probe.rs"
        atomic_text(probe_source, PROBE)
        probe_binary = output / "unwind-probe"
        probe_command = [resolved["rustc"], str(probe_source), "--crate-name", "cargo_lust_unwind_probe", "--edition", "2024", "-Zcodegen-backend=" + str(library), "-Cpanic=unwind", "-Copt-level=0", "-Coverflow-checks=on", "-Cdebug-assertions=on", "-o", str(probe_binary)]
        run(probe_command, output, env)
        print("Checking panic recovery and destructor execution; the caught test panic is expected.", file=sys.stderr)
        probe_stdout = run([str(probe_binary)], output, env, capture=True)
        if probe_stdout != "catch_unwind=1 drops=1":
            raise SetupError("The backend failed the panic/drop runtime probe.")

        metadata = {
            "schema_version": 1, "pin": pin, "rustc_verbose_version": rustc_version,
            "cargo_verbose_version": cargo_version, "cargo_lock_sha256": sha256(source / "Cargo.lock"),
            "bootstrap_workspace_sha256": sha256(output / "Cargo.toml"),
            "build_command": command, "backend_rustflags": env["RUSTFLAGS"],
            "sysroot": "the pinned toolchain's ordinary LLVM-built standard library",
            "artifact": {"path": str(library), "sha256": digest, "bytes": library.stat().st_size},
            "qualification": {"panic_strategy": "unwind", "probe_command": probe_command, "probe_stdout": probe_stdout, "full_project_coverage_proven": False},
        }
        atomic_text(output / "backend.json", json.dumps(metadata, indent=2) + "\n")
        exports = f"export RUSTUP_TOOLCHAIN={shlex.quote(pin['toolchain'])}\nexport LUST_CRANELIFT_BACKEND={shlex.quote(str(library))}\n"
        atomic_text(output / "env.sh", exports)
        print(f"Built and checked {library}")
        print(f"Source {pin['revision']}; toolchain {pin['toolchain']}; features unwinding")
        print(exports, end="")
        print(f"Or load both exports with: source {shlex.quote(str(output / 'env.sh'))}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", default=str(ROOT / "target" / "complement"), help="Owned build/install directory; default tools/cargo-lust/target/complement.")
    parser.add_argument("--skip-install", action="store_true", help="Require the pinned nightly and components to be installed already.")
    args = parser.parse_args()
    try:
        setup(args)
    except (SetupError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"backend setup failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
