#!/usr/bin/env bash
# K3 + K6 spike runner (Linux x86_64). Build outputs go to spikes/out/.
# Usage: LUST_TCC_HOME=/path/to/third_party/tcc spikes/run.sh [quick|full]
# Every probe prints PASS/FAIL lines and a RESULT line. This script keeps
# going after a failure and prints a summary count at the end.
set -u
cd "$(dirname "$0")/.." || exit 2
ROOT=$(pwd)
TCC_HOME=${LUST_TCC_HOME:-$ROOT/third_party/tcc}
TCC=$TCC_HOME/bin/tcc
TCC_LIB=$TCC_HOME/lib/tcc
OUT=$ROOT/spikes/out
MODE=${1:-full}
if [ "$MODE" = quick ]; then VECTORS=3000; BENCH_ITERS=200000; ROUNDS=3; else VECTORS=50000; BENCH_ITERS=2000000; ROUNDS=5; fi
mkdir -p "$OUT"
GCC_DIR=$(dirname "$(cc -print-libgcc-file-name)")
export RUSTC=${RUSTC:-rustc}
FAILURES=0
note() { echo; echo "== $* =="; }
# run_probe <label> <cmd...> : run, show output, count a failure on nonzero exit.
run_probe() {
    label=$1; shift
    echo "-- $label: $*"
    if "$@"; then echo "OK $label"; else rc=$?; echo "BAD $label exit=$rc"; FAILURES=$((FAILURES + 1)); fi
}
# build_or_fail <label> <cmd...> : like run_probe but quiet on success output.
build_or_fail() {
    label=$1; shift
    if "$@" >"$OUT/build-$label.log" 2>&1; then echo "BUILD-OK $label"; else
        echo "BUILD-FAIL $label:"; cat "$OUT/build-$label.log"; FAILURES=$((FAILURES + 1)); fi
}

note "environment"
uname -a; nproc
"$TCC" -v 2>&1 | head -1
cc --version | head -1
$RUSTC --version
ls -l "$TCC_HOME/lib/libtcc.a" "$TCC"

note "libtcc driver (same API sequence as src/tcc.rs)"
build_or_fail libtcc_driver cc -I"$TCC_HOME/include" spikes/libtcc_driver.c \
    "$TCC_HOME/lib/libtcc.a" -lm -ldl -lpthread -o "$OUT/libtcc_driver"
DRV="$OUT/libtcc_driver"

note "K3 atomics: tcc CLI, libtcc compile_string, cc reference"
build_or_fail atomics_tcc "$TCC" -w spikes/k3/atomics.c -lpthread -o "$OUT/atomics_tcc"
cp spikes/k3/atomics.c "$OUT/atomics.cstr"
build_or_fail atomics_libtcc "$DRV" "$TCC_LIB" exe "$OUT/atomics_libtcc" "$OUT/atomics.cstr" "-l:/usr/lib/x86_64-linux-gnu:pthread"
build_or_fail atomics_cc cc -O2 -w spikes/k3/atomics.c -lpthread -o "$OUT/atomics_cc"
for n in 2 4 8; do run_probe "atomics_tcc_threads$n" "$OUT/atomics_tcc" "$n" 50000; done
run_probe atomics_libtcc_threads4 "$OUT/atomics_libtcc" 4 50000
run_probe atomics_cc_threads4 "$OUT/atomics_cc" 4 50000
if "$TCC" -w spikes/k3/atomic128_probe.c -o "$OUT/atomic128_tcc" >"$OUT/atomic128_tcc.log" 2>&1; then
    echo "FAIL atomic128_tcc_reject (tcc accepted 16-byte atomics)"; run_probe atomic128_tcc "$OUT/atomic128_tcc"
else echo "PASS atomic128_tcc_reject:"; head -3 "$OUT/atomic128_tcc.log"; fi
build_or_fail atomic128_cc_latomic cc -O2 -w spikes/k3/atomic128_probe.c -latomic -o "$OUT/atomic128_cc"
run_probe atomic128_cc_latomic "$OUT/atomic128_cc"
build_or_fail atomic128_cc_cx16 cc -O2 -w -mcx16 spikes/k3/atomic128_probe.c -latomic -o "$OUT/atomic128_cx16"
run_probe atomic128_cc_cx16 "$OUT/atomic128_cx16"
if "$TCC" -w spikes/k3/atomic_ptr_probe.c -o "$OUT/atomic_ptr_tcc" >"$OUT/atomic_ptr_tcc.log" 2>&1; then
    echo "FAIL atomic_ptr_tcc_reject (tcc accepted pointer fetch_add)"; run_probe atomic_ptr_tcc "$OUT/atomic_ptr_tcc"
else echo "PASS atomic_ptr_tcc_reject:"; head -3 "$OUT/atomic_ptr_tcc.log"; fi
build_or_fail atomic_ptr_cc cc -O2 -w spikes/k3/atomic_ptr_probe.c -o "$OUT/atomic_ptr_cc"
run_probe atomic_ptr_cc "$OUT/atomic_ptr_cc"
echo "-- spin forms under tcc (compile+link+run per form; FAIL here is data, not a runner failure):"
for form in 1 2 3 4 5 6; do
    if "$TCC" -w -DSPIN_FORM="$form" spikes/k3/spin_probe.c -o "$OUT/spin$form" >"$OUT/spin$form.log" 2>&1 && "$OUT/spin$form" >/dev/null 2>&1; then
        echo "PASS spin_tcc_form$form"
    else echo "FAIL spin_tcc_form$form:"; head -2 "$OUT/spin$form.log"; fi
done
echo "-- spin forms under cc:"
for form in 1 2 3 4 5 6; do
    if cc -O2 -w -DSPIN_FORM="$form" spikes/k3/spin_probe.c -o "$OUT/spin_cc$form" >"$OUT/spin_cc$form.log" 2>&1 && "$OUT/spin_cc$form" >/dev/null 2>&1; then
        echo "PASS spin_cc_form$form"
    else echo "FAIL spin_cc_form$form:"; head -2 "$OUT/spin_cc$form.log"; fi
done

note "K3 TLS in executable"
build_or_fail tls_exe_tcc "$TCC" -w spikes/k3/tls_exe.c -lpthread -o "$OUT/tls_exe_tcc"
cp spikes/k3/tls_exe.c "$OUT/tls_exe.cstr"
build_or_fail tls_exe_libtcc "$DRV" "$TCC_LIB" exe "$OUT/tls_exe_libtcc" "$OUT/tls_exe.cstr" "-l:/usr/lib/x86_64-linux-gnu:pthread"
run_probe tls_exe_tcc "$OUT/tls_exe_tcc"
run_probe tls_exe_libtcc "$OUT/tls_exe_libtcc"

note "K3 TLS in shared object (libbase style), linked and dlopen"
build_or_fail tls_so_obj "$TCC" -w -c spikes/k3/tls_so.c -o "$OUT/tls_so.o"
echo "-- relocations in tcc tls_so.o:"; readelf -r "$OUT/tls_so.o" 2>&1 | head -30
build_or_fail tls_so_tccshared "$TCC" -w -shared spikes/k3/tls_so.c -o "$OUT/libtls_tcc.so"
cp spikes/k3/tls_so.c "$OUT/tls_so.cstr"
build_or_fail tls_so_libtccdll "$DRV" "$TCC_LIB" dll "$OUT/libtls_libtcc.so" "$OUT/tls_so.cstr"
build_or_fail tls_so_cc cc -O2 -shared -fPIC spikes/k3/tls_so.c -o "$OUT/libtls_cc.so"
echo "-- TLS segments:"; for so in libtls_tcc.so libtls_libtcc.so libtls_cc.so; do
    echo "$so:"; readelf -lW "$OUT/$so" 2>/dev/null | grep -i tls || echo "  (no TLS segment)"; done
# Linked exe (tcc) against each .so, rpath to out dir; distinct names via -l:file form is not
# portable in tcc, so link the .so by path.
build_or_fail tls_linked_tccso "$TCC" -w spikes/k3/tls_linked.c "$OUT/libtls_tcc.so" -Wl,-rpath="$OUT" -lpthread -o "$OUT/tls_linked_tccso"
build_or_fail tls_linked_ccso "$TCC" -w spikes/k3/tls_linked.c "$OUT/libtls_cc.so" -Wl,-rpath="$OUT" -lpthread -o "$OUT/tls_linked_ccso"
build_or_fail tls_linked_cc_all cc -O2 -w spikes/k3/tls_linked.c "$OUT/libtls_cc.so" -Wl,-rpath="$OUT" -lpthread -o "$OUT/tls_linked_cc_all"
run_probe tls_linked_exe-tcc_so-tcc "$OUT/tls_linked_tccso"
run_probe tls_linked_exe-tcc_so-cc "$OUT/tls_linked_ccso"
run_probe tls_linked_exe-cc_so-cc "$OUT/tls_linked_cc_all"
# libtcc overlay shape: exe from objects + library, as link_with_base does.
mkdir -p "$OUT/base_tls"
cp "$OUT/libtls_libtcc.so" "$OUT/base_tls/libbase.so"
build_or_fail tls_overlay_obj "$TCC" -w -c spikes/k3/tls_linked.c -o "$OUT/tls_linked.o"
build_or_fail tls_overlay_libtcc "$DRV" "$TCC_LIB" exe "$OUT/tls_overlay_libtcc" \
    "-o:-Wl,-rpath=$OUT/base_tls" "-l:$OUT/base_tls:base" "$OUT/tls_linked.o" "-l:/usr/lib/x86_64-linux-gnu:pthread"
run_probe tls_overlay_libtcc_base "$OUT/tls_overlay_libtcc"
# dlopen
build_or_fail tls_dlopen_exe "$TCC" -w spikes/k3/tls_dlopen.c -ldl -lpthread -o "$OUT/tls_dlopen"
run_probe tls_dlopen_tcc_so "$OUT/tls_dlopen" "$OUT/libtls_tcc.so"
run_probe tls_dlopen_cc_so "$OUT/tls_dlopen" "$OUT/libtls_cc.so"

note "K3 pthread-key TLS inside a tcc-built .so (the daemon-safe pattern)"
build_or_fail tls_key_so "$TCC" -w -shared spikes/k3/tls_key_so.c -lpthread -o "$OUT/libtls_key.so"
build_or_fail tls_linked_key "$TCC" -w spikes/k3/tls_linked.c "$OUT/libtls_key.so" -Wl,-rpath="$OUT" -lpthread -o "$OUT/tls_linked_key"
run_probe tls_linked_key_so "$OUT/tls_linked_key"
run_probe tls_dlopen_key_so "$OUT/tls_dlopen" "$OUT/libtls_key.so"

note "K3 libtcc in-memory mode (TCC_OUTPUT_MEMORY, as run_in_memory in src/tcc.rs)"
run_probe libtcc_run_tls_exe "$DRV" "$TCC_LIB" run - "$OUT/tls_exe.cstr" "-l:/usr/lib/x86_64-linux-gnu:pthread"
run_probe libtcc_run_atomics "$DRV" "$TCC_LIB" run - "$OUT/atomics.cstr" "-l:/usr/lib/x86_64-linux-gnu:pthread"

note "K3 pthread keys (portable TLS plan)"
build_or_fail pthread_key "$TCC" -w spikes/k3/pthread_key.c -lpthread -o "$OUT/pthread_key"
run_probe pthread_key "$OUT/pthread_key"

note "K3 __int128 (compile must fail under tcc, pass under cc)"
if "$TCC" -w spikes/k3/i128_probe.c -o "$OUT/i128_tcc" >"$OUT/i128_tcc.log" 2>&1; then
    echo "FAIL i128_tcc_reject (tcc accepted __int128)"; FAILURES=$((FAILURES + 1))
else echo "PASS i128_tcc_reject:"; head -3 "$OUT/i128_tcc.log"; fi
build_or_fail i128_cc cc -O2 -w spikes/k3/i128_probe.c -o "$OUT/i128_cc"
run_probe i128_cc "$OUT/i128_cc"

note "K3 u128 helpers vs rustc oracle ($VECTORS random vectors + edges)"
build_or_fail u128_oracle $RUSTC --edition 2024 -O -Awarnings spikes/k3/u128_oracle.rs -o "$OUT/u128_oracle"
"$OUT/u128_oracle" "$VECTORS" > "$OUT/u128_vectors.txt"
wc -l "$OUT/u128_vectors.txt"
build_or_fail u128_test_tcc "$TCC" -w -Ispikes/k3 spikes/k3/u128_test.c -o "$OUT/u128_test_tcc"
build_or_fail u128_test_cc cc -O2 -w -Ispikes/k3 spikes/k3/u128_test.c -o "$OUT/u128_test_cc"
run_probe u128_test_tcc "$OUT/u128_test_tcc" "$OUT/u128_vectors.txt"
run_probe u128_test_cc "$OUT/u128_test_cc" "$OUT/u128_vectors.txt"

note "K3 u128 benchmark (tcc code, cc -O2 same C, rustc native)"
build_or_fail u128_bench_tcc "$TCC" -w -Ispikes/k3 spikes/k3/u128_bench.c -o "$OUT/u128_bench_tcc"
build_or_fail u128_bench_cc cc -O2 -w -Ispikes/k3 spikes/k3/u128_bench.c -o "$OUT/u128_bench_cc"
build_or_fail u128_bench_rs $RUSTC --edition 2024 -O -Awarnings spikes/k3/u128_bench.rs -o "$OUT/u128_bench_rs"
run_probe u128_bench_tcc "$OUT/u128_bench_tcc" "$BENCH_ITERS"
run_probe u128_bench_cc "$OUT/u128_bench_cc" "$BENCH_ITERS"
run_probe u128_bench_rs "$OUT/u128_bench_rs" "$BENCH_ITERS"

note "K3 alignment and packed structs"
build_or_fail align_tcc "$TCC" -w spikes/k3/align.c -o "$OUT/align_tcc"
build_or_fail align_cc cc -O2 -w spikes/k3/align.c -o "$OUT/align_cc"
run_probe align_tcc "$OUT/align_tcc"
run_probe align_cc "$OUT/align_cc"
build_or_fail attr_variants_tcc "$TCC" -w spikes/k3/attr_variants.c -o "$OUT/attr_variants_tcc"
build_or_fail attr_variants_cc cc -O2 -w spikes/k3/attr_variants.c -o "$OUT/attr_variants_cc"
echo "-- attr_variants tcc:"; "$OUT/attr_variants_tcc"
echo "-- attr_variants cc:"; "$OUT/attr_variants_cc"
note "K6 build Rust shim as staticlib and cdylib"
SFLAGS="--edition 2024 -Awarnings -C opt-level=2"
build_or_fail shim_staticlib $RUSTC $SFLAGS --crate-type staticlib spikes/k6/shim.rs -o "$OUT/libshim.a"
build_or_fail shim_cdylib $RUSTC $SFLAGS --crate-type cdylib spikes/k6/shim.rs -o "$OUT/libshim.so"
ls -l "$OUT/libshim.a" "$OUT/libshim.so"
echo "-- staticlib sections of note:"; ar t "$OUT/libshim.a" | head -5
build_or_fail k6_main_obj "$TCC" -w -Ispikes/k6 -c spikes/k6/main.c -o "$OUT/k6_main.o"

note "K6 link path 1: tcc CLI + staticlib"
build_or_fail k6_tcc_static "$TCC" -w -Ispikes/k6 spikes/k6/main.c "$OUT/libshim.a" \
    -L"$GCC_DIR" -lpthread -ldl -lm -lgcc_s -o "$OUT/k6_tcc_static"
run_probe k6_tcc_static "$OUT/k6_tcc_static"

note "K6 tcc + staticlib without a shim panic (isolates the unwind fault)"
build_or_fail k6_tcc_static_nopanic "$TCC" -w -Ispikes/k6 spikes/k6/main_nopanic.c "$OUT/libshim.a" \
    -L"$GCC_DIR" -lpthread -ldl -lm -lgcc_s -o "$OUT/k6_tcc_static_nopanic"
run_probe k6_tcc_static_nopanic "$OUT/k6_tcc_static_nopanic"

note "K6 link path 2: libtcc API + staticlib"
build_or_fail k6_libtcc_static "$DRV" "$TCC_LIB" exe "$OUT/k6_libtcc_static" \
    "$OUT/k6_main.o" "$OUT/libshim.a" "-l:$GCC_DIR:gcc_s" \
    "-l:/usr/lib/x86_64-linux-gnu:pthread" "-l:/usr/lib/x86_64-linux-gnu:dl" "-l:/usr/lib/x86_64-linux-gnu:m"
run_probe k6_libtcc_static "$OUT/k6_libtcc_static"

note "K6 link path 3: tcc CLI + cdylib"
build_or_fail k6_tcc_cdylib "$TCC" -w -Ispikes/k6 spikes/k6/main.c -L"$OUT" -lshim \
    -Wl,-rpath="$OUT" -lpthread -ldl -o "$OUT/k6_tcc_cdylib"
run_probe k6_tcc_cdylib "$OUT/k6_tcc_cdylib"

note "K6 link path 4: libtcc API + cdylib"
build_or_fail k6_libtcc_cdylib "$DRV" "$TCC_LIB" exe "$OUT/k6_libtcc_cdylib" \
    "-o:-Wl,-rpath=$OUT" "$OUT/k6_main.o" "-l:$OUT:shim" "-l:/usr/lib/x86_64-linux-gnu:pthread"
run_probe k6_libtcc_cdylib "$OUT/k6_libtcc_cdylib"

note "K6 link path 5: system linker (cc) + tcc object + staticlib"
build_or_fail k6_cc_static cc "$OUT/k6_main.o" "$OUT/libshim.a" -lpthread -ldl -lm -o "$OUT/k6_cc_static"
run_probe k6_cc_static "$OUT/k6_cc_static"

note "K6 link path 6: system linker (cc) + tcc object + cdylib"
build_or_fail k6_cc_cdylib cc "$OUT/k6_main.o" -L"$OUT" -lshim -Wl,-rpath="$OUT" -lpthread -ldl -o "$OUT/k6_cc_cdylib"
run_probe k6_cc_cdylib "$OUT/k6_cc_cdylib"

note "K6 root cause: eh_frame in tcc objects and in the tcc linker product"
echo "-- FDE relocations in the tcc object (all .text + 0 means overlap):"
readelf -rW "$OUT/k6_main.o" | sed -n '/rela.eh_frame/,$p'
python3 spikes/k6/ehdr_check.py "$OUT/k6_tcc_static" || true
build_or_fail k6_main_cc_obj cc -w -Ispikes/k6 -c spikes/k6/main.c -o "$OUT/k6_main_cc.o"
build_or_fail k6_cc_static_ccobj cc "$OUT/k6_main_cc.o" "$OUT/libshim.a" -lpthread -ldl -lm -o "$OUT/k6_cc_static_ccobj"
run_probe k6_cc_static_ccobj "$OUT/k6_cc_static_ccobj"
if objcopy --remove-section .eh_frame --remove-section .rela.eh_frame "$OUT/k6_main.o" "$OUT/k6_main_noeh.o" 2>"$OUT/objcopy.log"; then
    echo "BUILD-OK k6_main_noeh_obj"
    build_or_fail k6_cc_static_noeh cc "$OUT/k6_main_noeh.o" "$OUT/libshim.a" -lpthread -ldl -lm -o "$OUT/k6_cc_static_noeh"
    run_probe k6_cc_static_noeh "$OUT/k6_cc_static_noeh"
    # Same stripped object through the tcc linker: still expected to abort,
    # which puts a second fault in the tcc linker's own eh_frame_hdr build.
    build_or_fail k6_tcc_static_noeh "$TCC" "$OUT/k6_main_noeh.o" "$OUT/libshim.a" -L"$GCC_DIR" -lpthread -ldl -lm -lgcc_s -o "$OUT/k6_tcc_static_noeh"
    run_probe k6_tcc_static_noeh "$OUT/k6_tcc_static_noeh"
    python3 spikes/k6/ehdr_check.py "$OUT/k6_cc_static_noeh" "$OUT/k6_tcc_static_noeh" || true
else echo "BUILD-FAIL k6_main_noeh_obj:"; cat "$OUT/objcopy.log"; FAILURES=$((FAILURES + 1)); fi
# tcc can skip eh_frame at the source with -fno-asynchronous-unwind-tables.
build_or_fail k6_main_tcc_noehflag_obj "$TCC" -w -fno-asynchronous-unwind-tables -Ispikes/k6 -c spikes/k6/main.c -o "$OUT/k6_main_tcc_noehflag.o"
if readelf -SW "$OUT/k6_main_tcc_noehflag.o" | grep -q eh_frame; then
    echo "FAIL noehflag_removes_eh_frame"; FAILURES=$((FAILURES + 1))
else echo "PASS noehflag_removes_eh_frame"; fi
build_or_fail k6_cc_static_noehflag cc "$OUT/k6_main_tcc_noehflag.o" "$OUT/libshim.a" -lpthread -ldl -lm -o "$OUT/k6_cc_static_noehflag"
run_probe k6_cc_static_noehflag "$OUT/k6_cc_static_noehflag"

note "K6 dlopen of cdylib from a tcc executable"
build_or_fail k6_dlopen_exe "$TCC" -w -Ispikes/k6 spikes/k6/dlopen_main.c -ldl -lpthread -o "$OUT/k6_dlopen"
run_probe k6_dlopen_cdylib "$OUT/k6_dlopen" "$OUT/libshim.so"

note "K6 allocator cross-free (observation only; contract forbids it)"
build_or_fail k6_crossfree cc -w -Ispikes/k6 spikes/k6/crossfree.c "$OUT/libshim.a" -lpthread -ldl -lm -o "$OUT/k6_crossfree"
echo "-- k6_crossfree:"; "$OUT/k6_crossfree"; echo "crossfree exit=$?"

note "K6 daemon layout: libbase.so (calls shim) + overlay exe via libtcc"
# Variant A: base built by tcc -shared with the staticlib inside.
mkdir -p "$OUT/daemonA" "$OUT/daemonB" "$OUT/daemonC"
build_or_fail baseA "$TCC" -w -Ispikes/k6 -shared spikes/k6/base.c "$OUT/libshim.a" \
    -L"$GCC_DIR" -lpthread -ldl -lm -lgcc_s -o "$OUT/daemonA/libbase.so"
# Variant B: base built by tcc -shared, depends on the cdylib.
build_or_fail baseB "$TCC" -w -Ispikes/k6 -shared spikes/k6/base.c -L"$OUT" -lshim \
    -Wl,-rpath="$OUT" -lpthread -o "$OUT/daemonB/libbase.so"
# Variant C: base built by cc -shared from a tcc object + staticlib.
build_or_fail baseC_obj "$TCC" -w -Ispikes/k6 -c spikes/k6/base.c -o "$OUT/baseC.o"
build_or_fail baseC cc -shared "$OUT/baseC.o" "$OUT/libshim.a" -lpthread -ldl -lm -o "$OUT/daemonC/libbase.so"
build_or_fail overlay_obj "$TCC" -w -c spikes/k6/overlay.c -o "$OUT/overlay.o"
for v in A B C; do
    d="$OUT/daemon$v"
    build_or_fail "overlay$v" "$DRV" "$TCC_LIB" exe "$d/overlay" "-o:-Wl,-rpath=$d" "-l:$d:base" "$OUT/overlay.o"
    run_probe "daemon_layout_$v" "$d/overlay"
    echo "-- ldd daemon$v/libbase.so:"; ldd "$d/libbase.so" 2>&1 | head -8
done

note "K6 link timing on ~200-export shim ($ROUNDS rounds, interleaved)"
python3 spikes/k6/gen_big.py "$OUT/big"
build_or_fail shim_big_static $RUSTC $SFLAGS --crate-type staticlib "$OUT/big/shim_big.rs" -o "$OUT/big/libshim_big.a"
build_or_fail shim_big_cdylib $RUSTC $SFLAGS --crate-type cdylib "$OUT/big/shim_big.rs" -o "$OUT/big/libshim_big.so"
build_or_fail shim_big_main_obj "$TCC" -w -I"$OUT/big" -c "$OUT/big/main_big.c" -o "$OUT/big/main_big.o"
ls -l "$OUT/big/libshim_big.a" "$OUT/big/libshim_big.so"
python3 spikes/k6/time_links.py "$OUT" "$TCC_HOME" "$ROUNDS" || FAILURES=$((FAILURES + 1))

note "summary"
echo "TOTAL_FAILURES=$FAILURES"
test "$FAILURES" -eq 0
