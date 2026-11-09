#!/usr/bin/env bash
set -euo pipefail
lust_script_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
exec python3 "$lust_script_dir/build-cranelift.py" "$@"
