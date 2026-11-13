#!/usr/bin/env bash
# K3 + K6 probes for macOS arm64. Companion to PROBES.md in this directory.
# This script has never run on a Mac. It mirrors spikes/run.sh but uses
# Mach-O spelling: dylib, -install_name, LC_RPATH via -Wl,-rpath, otool/nm.
#
# Usage on the Mac, from the repo root:
#   bash scripts/build-tcc.sh          # builds third_party/tcc (M0)
#   LUST_TCC_HOME=$PWD/third_party/tcc bash spikes/macos/run-macos.sh 2>&1 | tee /tmp/lust-macos-spikes.log
# Send the log back in full. Do not edit probe expectations to make it pass.
set -u
cd "$(dirname "$0")/../.." || exit 2
ROOT=$(pwd)
TCC_HOME=${LUST_TCC_HOME:-$ROOT/third_party/tcc}
TCC=$TCC_HOME/bin/tcc
TCC_LIB=$TCC_HOME/lib/tcc
OUT=$ROOT/spikes/out-macos
mkdir -p "$OUT"
FAILURES=0
note() { echo; echo "== $* =="; }
run_probe() {
    label=$1; shift
    echo "-- $label: $*"
    if "$@"; then echo "OK $label"; else rc=$?; echo "BAD $label exit=$rc"; FAILURES=$((FAILURES + 1)); fi
}
build_or_fail() {
    label=$1; shift
    if "$@" >"$OUT/build-$label.log" 2>&1; then echo "BUILD-OK $label"; else
        echo "BUILD-FAIL $label:"; cat "$OUT/build-$label.log"; FAILURES=$((FAILURES + 1)); fi
}

note "M0/M1 environment and hello world"
sw_vers 2>/dev/null; uname -m; sysctl -n machdep.cpu.brand_string 2>/dev/null
"$TCC" -v 2>&1 | head -1
cc --version | head -2
rustc --version
printf '#include <stdio.h>\nint main(void){printf("hello macho\\n");return 0;}\n' > "$OUT/hello.c"
build_or_fail hello "$TCC" "$OUT/hello.c" -o "$OUT/hello"
run_probe hello "$OUT/hello"
echo "-- load commands:"; otool -l "$OUT/hello" 2>&1 | grep -A2 "LC_RPATH\|LC_LOAD_DYLIB\|cmd LC_MAIN" | head -30

note "M2 atomics (spin hint check will FAIL on arm64 by design; it is x86-only)"
build_or_fail atomics_tcc "$TCC" -w spikes/k3/atomics.c -lpthread -o "$OUT/atomics_tcc"
for n in 2 4 8; do run_probe "atomics_threads$n" "$OUT/atomics_tcc" "$n" 50000; done

note "M3 spin hint candidates (compile only; PASS means the form compiles)"
printf 'int main(void){__builtin_arm_yield();return 0;}\n' > "$OUT/spin1.c"
printf 'int main(void){__asm__ volatile("yield");return 0;}\n' > "$OUT/spin2.c"
printf 'int main(void){__asm__ volatile("");return 0;}\n' > "$OUT/spin3.c"
for i in 1 2 3; do
    if "$TCC" -w -c "$OUT/spin$i.c" -o "$OUT/spin$i.o" >"$OUT/spin$i.log" 2>&1; then
        echo "PASS spin_form$i"; else echo "FAIL spin_form$i:"; head -2 "$OUT/spin$i.log"; fi
done

note "M4/M5 atomic128 and __int128 (both compiles must fail under tcc)"
if "$TCC" -w spikes/k3/atomic128_probe.c -o "$OUT/a128" >"$OUT/a128.log" 2>&1; then
    echo "FAIL atomic128_tcc_reject"; FAILURES=$((FAILURES + 1)); else echo "PASS atomic128_tcc_reject:"; head -3 "$OUT/a128.log"; fi
if "$TCC" -w spikes/k3/i128_probe.c -o "$OUT/i128" >"$OUT/i128.log" 2>&1; then
    echo "FAIL i128_tcc_reject"; FAILURES=$((FAILURES + 1)); else echo "PASS i128_tcc_reject:"; head -3 "$OUT/i128.log"; fi

note "M6 u128 helpers vs rustc oracle"
build_or_fail u128_oracle rustc --edition 2024 -O -Awarnings spikes/k3/u128_oracle.rs -o "$OUT/u128_oracle"
"$OUT/u128_oracle" 50000 > "$OUT/u128_vectors.txt"
build_or_fail u128_test "$TCC" -w -Ispikes/k3 spikes/k3/u128_test.c -o "$OUT/u128_test"
run_probe u128_test "$OUT/u128_test" "$OUT/u128_vectors.txt"

note "M7 alignment"
build_or_fail align_tcc "$TCC" -w spikes/k3/align.c -o "$OUT/align_tcc"
run_probe align_tcc "$OUT/align_tcc"
build_or_fail attr_variants "$TCC" -w spikes/k3/attr_variants.c -o "$OUT/attr_variants"
echo "-- attr_variants:"; "$OUT/attr_variants"

note "M8 TLS: exe, dylib (linked), dylib (dlopen)"
build_or_fail tls_exe "$TCC" -w spikes/k3/tls_exe.c -lpthread -o "$OUT/tls_exe"
run_probe tls_exe "$OUT/tls_exe"
# tcc Mach-O shared library flag may be -shared; if that fails, try -dynamiclib via -Wl.
if ! "$TCC" -w -shared spikes/k3/tls_so.c -o "$OUT/libtls.dylib" >"$OUT/tls_dylib.log" 2>&1; then
    echo "BUILD-FAIL tls_dylib -shared:"; cat "$OUT/tls_dylib.log"
    build_or_fail tls_dylib_wl "$TCC" -w -Wl,-dynamiclib spikes/k3/tls_so.c -o "$OUT/libtls.dylib"
else echo "BUILD-OK tls_dylib"; fi
echo "-- sections in libtls.dylib (look for __thread_data/__thread_vars):"
otool -l "$OUT/libtls.dylib" 2>&1 | grep -i "sectname\|__thread" | head -20
nm "$OUT/libtls.dylib" 2>&1 | head -20
build_or_fail tls_linked "$TCC" -w spikes/k3/tls_linked.c "$OUT/libtls.dylib" -Wl,-rpath,"$OUT" -lpthread -o "$OUT/tls_linked"
run_probe tls_linked "$OUT/tls_linked"
build_or_fail tls_dlopen "$TCC" -w spikes/k3/tls_dlopen.c -lpthread -o "$OUT/tls_dlopen"
run_probe tls_dlopen "$OUT/tls_dlopen" "$OUT/libtls.dylib"

note "M9 pthread keys"
build_or_fail pthread_key "$TCC" -w spikes/k3/pthread_key.c -lpthread -o "$OUT/pthread_key"
run_probe pthread_key "$OUT/pthread_key"

note "libtcc driver"
build_or_fail libtcc_driver cc -I"$TCC_HOME/include" spikes/libtcc_driver.c \
    "$TCC_HOME/lib/libtcc.a" -lm -o "$OUT/libtcc_driver"
DRV="$OUT/libtcc_driver"

note "M10 shim link paths"
SFLAGS="--edition 2024 -Awarnings -C opt-level=2"
# shellcheck disable=SC2086
build_or_fail shim_static rustc $SFLAGS --crate-type staticlib spikes/k6/shim.rs -o "$OUT/libshim.a"
# shellcheck disable=SC2086
build_or_fail shim_cdylib rustc $SFLAGS --crate-type cdylib spikes/k6/shim.rs -o "$OUT/libshim.dylib"
ls -l "$OUT"/libshim.*
build_or_fail k6_obj "$TCC" -w -Ispikes/k6 -c spikes/k6/main.c -o "$OUT/k6_main.o"
build_or_fail m10_tcc_static "$TCC" -w -Ispikes/k6 spikes/k6/main.c "$OUT/libshim.a" -lpthread -o "$OUT/m10_tcc_static"
run_probe m10_tcc_static "$OUT/m10_tcc_static"
build_or_fail m10_libtcc_static "$DRV" "$TCC_LIB" exe "$OUT/m10_libtcc_static" "$OUT/k6_main.o" "$OUT/libshim.a"
run_probe m10_libtcc_static "$OUT/m10_libtcc_static"
build_or_fail m10_tcc_cdylib "$TCC" -w -Ispikes/k6 spikes/k6/main.c -L"$OUT" -lshim -Wl,-rpath,"$OUT" -lpthread -o "$OUT/m10_tcc_cdylib"
run_probe m10_tcc_cdylib "$OUT/m10_tcc_cdylib"
build_or_fail m10_libtcc_cdylib "$DRV" "$TCC_LIB" exe "$OUT/m10_libtcc_cdylib" "-o:-Wl,-rpath,$OUT" "$OUT/k6_main.o" "-l:$OUT:shim"
run_probe m10_libtcc_cdylib "$OUT/m10_libtcc_cdylib"
build_or_fail m10_cc_static cc "$OUT/k6_main.o" "$OUT/libshim.a" -lpthread -framework Security -framework CoreFoundation -o "$OUT/m10_cc_static"
run_probe m10_cc_static "$OUT/m10_cc_static"
build_or_fail m10_cc_cdylib cc "$OUT/k6_main.o" -L"$OUT" -lshim -Wl,-rpath,"$OUT" -lpthread -o "$OUT/m10_cc_cdylib"
run_probe m10_cc_cdylib "$OUT/m10_cc_cdylib"
build_or_fail m10_dlopen "$TCC" -w -Ispikes/k6 spikes/k6/dlopen_main.c -lpthread -o "$OUT/m10_dlopen"
run_probe m10_dlopen "$OUT/m10_dlopen" "$OUT/libshim.dylib"

note "M11 daemon layout: libbase.dylib (cdylib wall) + overlay via libtcc"
mkdir -p "$OUT/daemonB"
build_or_fail baseB_obj "$TCC" -w -Ispikes/k6 -c spikes/k6/base.c -o "$OUT/baseB.o"
build_or_fail baseB_cc cc -dynamiclib -install_name "@rpath/libbase.dylib" "$OUT/baseB.o" \
    -L"$OUT" -lshim -Wl,-rpath,"$OUT" -o "$OUT/daemonB/libbase.dylib"
build_or_fail overlay_obj "$TCC" -w -c spikes/k6/overlay.c -o "$OUT/overlay.o"
build_or_fail overlayB "$DRV" "$TCC_LIB" exe "$OUT/daemonB/overlay" \
    "-o:-Wl,-rpath,$OUT/daemonB" "-l:$OUT/daemonB:base" "$OUT/overlay.o"
echo "-- overlay load commands:"; otool -l "$OUT/daemonB/overlay" 2>&1 | grep -A3 "LC_RPATH\|LC_LOAD_DYLIB" | head -30
run_probe daemon_layout_macos "$OUT/daemonB/overlay"
echo "-- retry with DYLD_LIBRARY_PATH if the direct run failed:"
DYLD_LIBRARY_PATH="$OUT/daemonB:$OUT" "$OUT/daemonB/overlay" || true

note "M13 link timing: run the Linux timer paths by hand (5 rounds, median)"
echo "Adapt spikes/k6/time_links.py: replace libshim_big.so with .dylib and drop -lgcc_s."
echo "TOTAL_FAILURES=$FAILURES"
