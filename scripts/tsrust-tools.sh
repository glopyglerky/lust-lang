#!/usr/bin/env bash
# tsrust-tools.sh [--refresh | --make-ref]: gate lane for the ts-rust fork targets that check under
# lust (scripts/tsrust_tools.py): the codegen tools (pinned copy), the fork libs and the tsgo bin
# (pinned fork tree). lust checks both tool crates (lib and bin), builds both bins, and their runs
# on the fork's schema and catalog inputs must equal the Cargo builds' (tsrust_parity TOOL_RUNS);
# it checks the fork libs, builds tsgo, and tsgo's fixture runs and the fork's integration test
# bins must equal the Cargo reference. The tool source is the pinned copy bench/tsrust-tools (gate
# boxes have no fork submodule); --refresh copies it again from third_party/ts-rust. Needs lust
# built (scripts/build.sh). Last line:
#   tsrust-tools: <ok> passed, <bad> failed
# --make-ref (once per fork pin, here): Cargo-build the tsgo reference of the pin on a boat
# sandbox and store it as <pin>.tsgo-ref.tzst in FORK_TARS, where remote-gate.sh finds it.
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
export LUST_TCC_HOME=${LUST_TCC_HOME:-/home/theo/.t3/projects/lust-lang/third_party/tcc}
if [ "${1:-}" = --make-ref ]; then
  ft=${FORK_TARS:-/home/theo/lust-scratch/tsrust-full/fork-tars}
  pin=$(git -C "$root" ls-tree HEAD third_party/ts-rust | awk '{print $3}')
  [ -s "$ft/$pin.tgz" ] || { echo "tsrust-tools: no fork tar $ft/$pin.tgz (a gate makes it)"; exit 2; }
  cd "$root" && "${BOAT_RUN:-/tmp/lust-wt/boat-run.sh}" --hours 2 --up "$ft/$pin.tgz" --fetch tsgo-ref -- \
    "mkdir -p ~/fork-tar && cp ~/up/$pin.tgz ~/fork-tar/ && TSRUST_FORK_PIN=$pin python3 scripts/tsrust_tools.py --make-ref ~/w/tsgo-ref" || exit $?
  mv "boat-out/tsgo-ref/$pin.tsgo-ref.tzst" "$ft/" && echo "tsrust-tools: stored $ft/$pin.tsgo-ref.tzst"
  exit $?
fi
exec python3 "$root/scripts/tsrust_tools.py" "$@"
