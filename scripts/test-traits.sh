#!/usr/bin/env bash
# Trait corpus runner (ARCH 9.3 runner table): selection,
# coherence, and inference cases in tests/traits/, plus the
# minimal generic pipeline fixtures in tests/mgp/.
# Usage: scripts/test-traits.sh [filter]
# Modes and knobs: see scripts/test-corpus-lib.sh.
set -uo pipefail
# shellcheck source=scripts/test-corpus-lib.sh
source "$(dirname "$0")/test-corpus-lib.sh"

corpus_init "${1:-}"
corpus_pass_dir tests/traits/pass
corpus_fail_dir tests/traits/fail
corpus_pass_dir tests/mgp/pass
corpus_finish test-traits
