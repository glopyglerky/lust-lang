#!/usr/bin/env bash
# Driver tests: daemon sessions, slot and unit reuse, warm/cold
# equality (scripts/test-driver.py). Run after `cargo build --release`.
# This wrapper exists so the landing gate can run the suite with
# script_step, like every other scripts/*.sh suite. The Python file
# holds the cases and prints the "test-driver: N passed, M failed"
# summary as its last line.
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
exec python3 "$root/scripts/test-driver.py"
