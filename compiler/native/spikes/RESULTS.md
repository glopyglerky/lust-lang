# K3 and K6 spike results

Date: 2026-10-12. Platform: Linux x86_64 (Debian, kernel 6.12, 24 cores).
Compilers: pinned tcc 0.9.28rc at tinycc commit 43c7708b
(`scripts/build-tcc.sh`), rustc 1.99.0, GCC 14.2.0 (`cc`).

Run everything again:

```sh
export LUST_TCC_HOME=/home/theo/.t3/projects/lust-lang/third_party/tcc
ulimit -c 0
bash spikes/run.sh quick    # 3k vectors, 3 timing rounds, about 1 minute
bash spikes/run.sh full     # 50k vectors, 5 timing rounds, 2M-iter benches
```

Outputs go to `spikes/out/` (ignored by git). Each probe prints PASS and
FAIL lines and one RESULT line. The runner keeps going after a failure and
ends with `TOTAL_FAILURES`. The full run used for the numbers below ended
with `TOTAL_FAILURES=19`. Every one of those 19 is a known tcc defect that
this document records, plus check `lockfree_macros_present` (4 runs), which
fails by design to prove the macro absence. No probe bug is in the count.

The libtcc API path is probed by `spikes/libtcc_driver.c`. It uses the same
call sequence as `src/tcc.rs`: `tcc_new`, `tcc_set_error_func`,
`tcc_set_lib_path`, `tcc_set_options` with `-w`, `tcc_set_output_type`,
then `tcc_compile_string` or `tcc_add_file` plus `tcc_add_library`, then
`tcc_output_file`. Its `run` mode uses `TCC_OUTPUT_MEMORY`,
`tcc_relocate`, and `tcc_get_symbol`, as `run_in_memory` in `src/tcc.rs`
does, without the fork.

## Answer table

| # | Question | Answer | Evidence | Number |
|---|---|---|---|---|
| K3-1 | C11 atomics, widths 8/16/32/64 plus bool, usize, pointer. Load, store, exchange, fetch add/sub/and/or/xor, CAS strong and weak, each with relaxed/consume/acquire/release/acq_rel/seq_cst where legal. | YES, all pass, all lock-free. | `k3/atomics.c`, tcc CLI, libtcc compile_string, cc reference | Stress exact at 2, 4, and 8 pthreads, 50000 iterations each: fetch_add and CAS counters equal threads times iterations. Acquire/release message passing passes. |
| K3-2 | `atomic_thread_fence` and `atomic_signal_fence`, all orders. | YES. | `k3/atomics.c` check `fences_all_orders` | Compiles and runs under tcc, libtcc, and in-memory mode. |
| K3-3 | `ATOMIC_*_LOCK_FREE` macros. | NO. The macros do not exist in tcc `stdatomic.h`. | `k3/atomics.c` check `lockfree_macros_present` | Compile of the macro names gives "undeclared". `atomic_is_lock_free` works and returns true for 1, 2, 4, and 8 bytes. |
| K3-4 | Fetch ops on `_Atomic(T *)`. | NO in tcc. Compile error. In cc the arithmetic is in bytes, not elements. | `k3/atomic_ptr_probe.c` | tcc: "integral or integer-sized pointer target type expected". cc: fetch_add of 2 moves the pointer by 2 bytes. Pointer load, store, exchange, and CAS do compile in tcc. |
| K3-5 | 128-bit atomics. | NO in tcc. Compile error for size over 8. | `k3/atomic128_probe.c` | cc with `-latomic` gives correct CAS with lock_free=0. Rust has no stable `AtomicU128`. This blocks nothing. |
| K3-6 | Spin hint for `spin_loop`. | YES via inline asm. NO via builtin. | `k3/spin_probe.c` forms 1 to 6 | Under tcc: form 1 `__builtin_ia32_pause` fails the link (unresolved reference), form 2 `xmmintrin.h` is absent, forms 3 (`pause`), 4 (`rep nop`), and 5 (signal fence) pass. Emit `__asm__ volatile("pause")` on x86_64. |
| K3-7 | `_Thread_local` and `__thread` in an executable. | YES. | `k3/tls_exe.c`, tcc CLI and libtcc | Main keeps 5/7/11, worker sees the init values and writes 99/88/77. |
| K3-8 | `_Thread_local` in a shared object that tcc builds, linked at load time (the libbase shape). | NO. It silently aliases the exe TLS. Opus B9 is correct. | `k3/tls_so.c` plus `k3/tls_linked.c`, tcc `-shared` and libtcc DLL plus libtcc overlay exe | `flag_addr()` in the .so equals the address of the exe variable. Write 77 in the exe, the .so reads 77. Write 41 in the .so, the exe reads 41. Init values are wrong: big reads 55, static reads 0. Per-thread isolation only looks right because both sides use the exe thread block. |
| K3-9 | dlopen of a tcc-built .so that defines TLS. | NO. It segfaults. | `k3/tls_dlopen.c` with `libtls_tcc.so` | Exit 139 (SIGSEGV) before the first check. The dlopen host exe has no TLS variable of its own, so the aliased offset lands outside its thread block. |
| K3-10 | A cc-built .so with TLS, linked into or dlopen'd from a tcc exe. | YES. | `k3/tls_linked.c` and `k3/tls_dlopen.c` with `libtls_cc.so` | All checks pass. The dynamic loader gives the .so its own TLS. The fault is in tcc codegen for `-shared`, not in the tcc exe as a host. |
| K3-11 | libtcc in-memory mode (`-run`) with TLS. | NO. Hard error at relocate time. | `libtcc_driver.c` run mode with `k3/tls_exe.c` | `tcc: error: thread-local storage not supported with -run`. Atomics in the same mode pass every functional check. |
| K3-12 | pthread keys: per-thread values and destructors, in an exe and inside a tcc-built .so. | YES. | `k3/pthread_key.c`, `k3/tls_key_so.c` with `tls_linked.c` and `tls_dlopen.c` | Destructor runs once per worker (4 calls for 4 workers). The .so form passes the full alias test that `_Thread_local` fails. |
| K3-13 | `__int128`. | NO. | `k3/i128_probe.c` | tcc: `'__int128' undeclared`. cc accepts the same file. |
| K3-14 | Two-word u128/i128 helpers match rustc. | YES. | `k3/u128.h`, `k3/u128_oracle.rs`, `k3/u128_test.c` | 50432 vectors (50000 random plus the edge grid), zero mismatches in add, sub, mul, unsigned and signed div and rem, shifts 0 to 127, comparisons, and decimal format. Same result under tcc and cc. |
| K3-15 | u128 helper cost under tcc codegen. | Numbers below in "u128 benchmark". | `k3/u128_bench.c`, `k3/u128_bench.rs` | add 11.75 ns vs rustc 1.00 ns. div 1673 ns vs 10.35 ns. Format 83943 ns vs 38.33 ns. |
| K3-16 | `_Alignas` on statics to 4 MiB. | YES. | `k3/align.c` checks `align_static_*` | 16, 64, 4096, and 4194304 all hold at run time. |
| K3-17 | 4 MiB-aligned heap allocation (the ts-rust jemalloc case). | YES, three ways. | `k3/align.c` checks `alloc_*_4MiB` | `posix_memalign`, `aligned_alloc`, and manual over-align with a stored base pointer all give 4 MiB-aligned, writable memory. |
| K3-18 | `aligned_alloc` declaration under tcc. | Hazard. It needs an explicit declaration in lust runtime C. | `k3/align.c` header note | tcc predefines `__STDC_VERSION__` as 199901L. glibc then hides the C11 `aligned_alloc` declaration. Without a declaration, tcc truncates the pointer return to int and the program segfaults in `free`. |
| K3-19 | Packed structs. | `__attribute__((packed))` NO. `#pragma pack` YES. | `k3/align.c`, `k3/attr_variants.c` | Attribute form: size 24, field d at 16 (same as natural). Pragma form: size 15, offsets 1, 5, 7. Byte image and unaligned access through the pragma form are correct in tcc. |
| K3-20 | `__attribute__((aligned(N)))` on a struct or typedef. | NO. tcc ignores it in every placement. | `k3/attr_variants.c` A1 to A3 | Size 1, align 1 for a 32-byte request. cc gives 32 and 32. |
| K3-21 | `_Alignas(N)` on a struct member, and a 16-byte aligned u128 type. | YES. | `k3/align.c` checks `align_struct_u128_field`, `align_struct_u128a16`, `alignas_member_32` | `struct { _Alignas(16) uint64_t lo; uint64_t hi; }` has size 16 and align 16. A following field sits at offset 16 in the parent. This is the u128 type shape codegen must emit. The bare two-word struct has align 8, which is wrong for Rust u128 layout. |
| K3-22 | `_Alignas` on stack locals above 16 bytes. | NO. Not reliable at 32 and above. | `k3/align.c` local checks, `k3/attr_variants.c` | Align 16 holds. At 64 and above the address residue varies between runs of the same binary (64 gave residues 0, 16, 32, 48). tcc does not realign the stack. |
| K6-1 | Rust staticlib, final link by tcc CLI. Semantic tests: caught internal panic, Rust TLS, Rust threads, callbacks Rust to C, allocator buffers. | FAIL. | `k6/main.c` as `k6_tcc_static` | Aborts at the panic test: `thread panicked while processing panic`. Exit 134. |
| K6-2 | Same, final link by libtcc API. | FAIL, same way. | `k6_libtcc_static` | Exit 134. The CLI and the library share the fault. |
| K6-3 | Same staticlib, shim code that never panics. | FAIL at Rust thread spawn. | `k6/main_nopanic.c` as `k6_tcc_static_nopanic` | `shim_add` and Rust TLS from C threads pass. `shim_spawn_sum_nopanic` aborts: panic inside `std/src/thread/local.rs` on the Rust-spawned thread, then a double panic. |
| K6-4 | Rust cdylib, final link by tcc CLI. | PASS, all tests. | `k6_tcc_cdylib` | Caught panic returns status 1 and message `shim internal boom 42`. Spawn sum 410. Callbacks, buffer ownership, TLS all pass. |
| K6-5 | Rust cdylib, final link by libtcc API. | PASS, all tests. | `k6_libtcc_cdylib` | Same numbers as K6-4. |
| K6-6 | dlopen of the cdylib from a tcc exe. | PASS. | `k6/dlopen_main.c` | Add, caught panic, and spawn all pass. |
| K6-7 | System linker final link: tcc object plus staticlib. | FAIL as written. | `k6_cc_static`, `k6_cc_cdylib` | GNU ld stops: `.eh_frame_hdr refers to overlapping FDEs`. Root cause in the next section. |
| K6-8 | System linker with the eh_frame problem removed (three variants). | PASS, all tests. | `k6_cc_static_ccobj`, `k6_cc_static_noeh`, `k6_cc_static_noehflag` | A cc-built main object, a tcc object with `.eh_frame` stripped by objcopy, and a tcc object built with `-fno-asynchronous-unwind-tables` all link and pass. |
| K6-9 | tcc linker with a clean (eh-stripped or cc-built) main object plus staticlib. | FAIL. | `k6_tcc_static_noeh` | Still aborts at the panic test. The tcc linker product has 2326 FDEs in `.eh_frame` but `fde_count` 1904 in `.eh_frame_hdr`. 422 frames are missing from the search table. |
| K6-10 | Daemon layout A: tcc `-shared` libbase with the staticlib linked inside, overlay exe by libtcc. | FAIL. | `k6/base.c`, `k6/overlay.c`, `daemonA` | Exit 134. Same double panic. |
| K6-11 | Daemon layout B: tcc `-shared` libbase that depends on the cdylib, overlay exe by libtcc. | PASS. | `daemonB` | `base_compute(32)` returns 1442 (32 plus 1000 for the caught panic plus 410 spawn sum). |
| K6-12 | Daemon layout C: cc `-shared` libbase from a tcc object plus the staticlib, overlay exe by libtcc. | PASS. | `daemonC` | Same 1442. Works because this base object has one function, so its single FDE cannot overlap. Do not read this as a general pass. |
| K6-13 | Cross-free: C `free` on a shim buffer, shim free on a C buffer. | No crash on this platform. Still forbid it by contract. | `k6/crossfree.c` | Exit 0 both ways. Rust default allocator is the system allocator here. With jemalloc or mimalloc as the Rust global allocator this can corrupt the heap. The boundary rule in ARCH 3.7 stands: shim memory is freed only by the shim. |
| K6-14 | Link time at realistic size (200-export shim). | Numbers below in "link timing". | `k6/gen_big.py`, `k6/time_links.py` | libtcc plus cdylib 3.5 ms. cc plus staticlib 195.3 ms. tcc plus staticlib 442 ms and the product aborts. |

Artifact sizes, for reference: `libshim.a` 23383228 bytes,
`libshim.so` 4571240 bytes, `libshim_big.a` 23681438 bytes,
`libshim_big.so` 4640760 bytes. These are rustc default builds with debug
info. Absolute sizes will change with strip and profile choices.

## Root causes found

### TLS in a tcc-built shared object uses the exe model

The tcc object for `tls_so.c` carries `R_X86_64_TPOFF32` relocations. That
is the local-exec model: load from `%fs` plus a fixed offset. A shared
object needs the dynamic model (global-dynamic or initial-exec through
`__tls_get_addr` or a GOT TPOFF slot) so the loader gives it a block of its
own. The .so even has a correct `PT_TLS` segment (20 bytes), but the code
never uses it. So the .so reads and writes the exe thread block at .so
offsets. With an exe TLS variable at the same offset, the two variables
are one variable. With no exe TLS variable, the offset can fall outside
the block and the access segfaults. This is silent memory corruption in
the daemon layout. There is no warning at build time.

In-memory mode is at least loud: `tcc_relocate` returns the error
`thread-local storage not supported with -run`.

### tcc codegen writes overlapping FDEs

For each function in a unit, tcc emits one `.eh_frame` FDE. Each FDE
initial-location relocation in the object refers to `.text + 0`:

```text
Relocation section '.rela.eh_frame' contains 3 entries:
00000020  R_X86_64_PC32  .text + 0
00000044  R_X86_64_PC32  .text + 0
00000068  R_X86_64_PC32  .text + 0
```

(from `k6_main.o`, functions at .text offsets 0, 0x44, and 0xa6, with FDE
lengths 0x44, 0x62, and 0x83a.). Every FDE claims to start at the same
address. GNU ld refuses the unit at final link. A unit with one function
links fine, which is why small probes before this spike did not see it.

Fix on the lust side: pass `-fno-asynchronous-unwind-tables` to tcc. The
probe `k6_main_tcc_noehflag.o` then has no `.eh_frame` at all, and the cc
final link passes all tests. Lust C frames never unwind (ARCH 7.2 uses a
flag), so the tables have no value. The option is a flag in
`libtcc.c` (`asynchronous-unwind-tables`, around line 1728) and
`unwind_tables` defaults to 1. lust should set it off in `src/tcc.rs` for
both the CLI calls and `tcc_set_options`. This spike does not change the
compiler. Wave 2 should.

Fix on the tcc side, if anyone wants it: the FDE relocation in `tccdbg.c`
needs the function offset as the addend. The pinned source is ours to
patch, but the flag above makes the patch unnecessary for lust.

### The tcc linker drops FDEs from the search table

With a clean main object, the tcc-linked exe still aborts on a caught
Rust panic. `k6/ehdr_check.py` compares the FDE count in `.eh_frame` with
the count stored in `.eh_frame_hdr`:

```text
EHDR k6_tcc_static       fdes_in_eh_frame=2329 fde_count_in_hdr=1907 overlaps=0
EHDR k6_tcc_static_noeh  fdes_in_eh_frame=2326 fde_count_in_hdr=1904 overlaps=0
EHDR k6_cc_static_noeh   fdes_in_eh_frame=2327 fde_count_in_hdr=2327 overlaps=0
```

The unwinder binary-searches `.eh_frame_hdr`. Frames past index 1907 are
unfindable. A panic whose return address sits in one of those frames
cannot start unwinding, and Rust reports a double panic and aborts. This
fault is in the linker only. No input hygiene on our side removes it. The
Rust thread spawn failure in K6-3 is a second symptom in the same link
path and it occurs with no user panic at all. The tcc linker also needs
the GCC runtime for the archive (`-L<gcc dir> -lgcc_s`); without it the
link fails with unresolved `_Unwind_*` names. With it, the link succeeds
and the product is wrong.

Conclusion: **the tcc linker must never be the final linker for a Rust
staticlib.** Link success proves nothing in that path.

## u128 benchmark

Method: 2M iterations for add, mul, shift, and compare. 100001 iterations
for div and format. Inputs cycle through 4096 pre-generated random pairs.
One process per implementation, one measurement per op (the box runs other
agents, so treat values under 20 percent apart as equal). `tcc` and
`cc -O2` run the same `u128.h` code. Sinks match across all three C runs.

| Op | u128.h under tcc | u128.h under cc -O2 | rustc native u128 | tcc helpers vs rustc |
|---|---|---|---|---|
| add | 11.75 ns | 1.07 ns | 1.00 ns | 11.8x |
| mul | 21.01 ns | 6.40 ns | 1.48 ns | 14.2x |
| div | 1672.91 ns | 215.61 ns | 10.35 ns | 161.6x |
| shl by variable | 14.97 ns | 1.13 ns | 2.71 ns | 5.5x |
| format decimal | 83942.59 ns | 16216.58 ns | 38.33 ns | 2190x |
| cmp | 12.04 ns | 1.13 ns | 0.85 ns | 14.2x |

Read this correctly. Most of the add and mul gap is tcc codegen in
general, which the whole dev binary pays on every type. The two real
algorithm problems are div and format. The helper div is a 128-step
restoring division. The helper format does about 39 of those divisions
per number. Before wave 2 uses these helpers for real, replace the
division with a normalized two-limb algorithm (Knuth D on 64-bit limbs,
with a fast path for a single-limb divisor), and give format a
divide-by-10^19 path that does one division per 19 digits. That work is
small and it removes the only three-digit slowdowns. Correctness is
already proven; keep the oracle test as the gate for the faster version.

Signed i128 div and rem in the helpers match rustc wrapping semantics,
including `i128::MIN / -1`, which wraps to `i128::MIN` in both the oracle
(`wrapping_div`) and the helper. The lust checker must still emit the
Rust panic for that case, and for division by zero, before the call. The
helpers assume a nonzero divisor.

Layout rule that K3-21 adds: the C type for Rust u128 and i128 must be
`struct { _Alignas(16) uint64_t lo; uint64_t hi; }`. The plain two-word
struct has size 16 but align 8. In arrays, in struct fields after an odd
field, and in `Option<u128>` style layouts, align 8 gives the wrong
offsets. `u128.h` in this spike uses the plain struct for the arithmetic
proof. The codegen type must carry the `_Alignas(16)` member form.

## Link timing

Method: the shim grows to 200 exported functions plus one catch function
(`k6/gen_big.py`). `k6/time_links.py` runs each link path 5 times,
interleaved round by round, and reports the median wall time of the link
command. The tcc CLI rows compile one small main file inside the timed
command. The libtcc and cc rows start from a pre-built tcc object, which
matches how lust links (objects first, final link second). After timing,
each product runs once. The `run` column is that product's own RESULT
line.

| Path | Median link | Exe bytes | Run |
|---|---|---|---|
| libtcc API + cdylib | 3.5 ms | 27290 | PASS |
| tcc CLI + cdylib | 5.8 ms | 27250 | PASS |
| cc + cdylib | 29.2 ms | 45184 | PASS |
| cc + staticlib | 195.3 ms | 5536912 | PASS |
| tcc CLI + staticlib | 442.2 ms | 2071149 | abort, signal 6 |
| libtcc API + staticlib | 448.4 ms | 2071117 | abort, signal 6 |

The fast paths are also the correct paths. The staticlib through the
system linker costs about 195 ms at this size and pulls the used part of
Rust std into a 5.5 MB exe. The cdylib paths cost 4 to 6 ms and leave the
Rust code in one shared file that every overlay reuses.

## Recommendations for ARCH and wave 2

### atomics (ARCH 3.2, 7.3): as written, with three spelling rules

1. Emit C11 atomics from `stdatomic.h` for bool and 8 to 64-bit ints and
   usize. All widths are lock-free on Linux x86_64. Map Rust orderings to
   C11 orderings one to one, as 7.3 says.
2. Never emit the `ATOMIC_*_LOCK_FREE` macros. They do not exist in the
   tcc header. If code needs lock-free status at run time, call
   `atomic_is_lock_free`. Rust const contexts that ask about lock-free
   widths have fixed answers per target; hard-code them.
3. Do pointer atomic read-modify-write on the `uintptr_t` bit pattern
   with explicit scaling by element size. tcc rejects fetch ops on
   pointer-typed atomics, and C pointer fetch arithmetic counts bytes.
   Pointer load, store, exchange, and CAS on `_Atomic(T *)` are fine.
4. `spin_loop` on x86_64 emits `__asm__ volatile("pause")`. Do not emit
   `__builtin_ia32_pause` (link fails) and do not include `xmmintrin.h`
   (absent).
5. 128-bit atomics do not exist in tcc and do not exist in stable Rust.
   Nothing to do.

### TLS (ARCH 7.2, 7.3): no C TLS anywhere, not only in shared objects

The safe rule for the daemon libbase plus overlay layout is stronger than
"no TLS in the .so":

1. Lust-generated C and the lust runtime C must not declare
   `_Thread_local` or `__thread` variables at all. In an exe they work,
   in a tcc-built .so they corrupt memory in silence, under dlopen they
   can segfault, and in libtcc memory mode they are a hard error. Lust
   code moves between these four shapes by design, so one rule must
   cover all of them.
2. Per-thread state goes through pthread keys, one key per
   `thread_local!` static, as 7.3 already says, and the 7.2 panic state
   must use the same rule. The `lust_tls_panic` struct in 7.2 must not be
   a C `_Thread_local`. Use the Opus shape: one plain global atomic
   counter as the fast path ("some thread is panicking"), and the
   per-thread payload behind a pthread key on the slow path.
3. TLS in a cc-built or rustc-built shared library is fine, including
   when the host exe is a tcc exe. The ban covers only code that tcc
   compiles. Rust `thread_local!` inside the shim cdylib is safe.
4. Add a build-time guard so this stays true: after linking libbase,
   assert that `readelf -r libbase.so` shows no `R_X86_64_TPOFF32` (or
   `TPREL` on arm64) relocations from lust objects, and keep one
   regression probe in this suite shape. A silent alias must never reach
   a user.

### alignment and packed (ARCH 3.2, 4.4): emit pragma and _Alignas forms only

1. Packed types: emit `#pragma pack(push, 1)` around the struct. Never
   emit `__attribute__((packed))`. tcc accepts the syntax and ignores it,
   which gives wrong layouts with no error.
2. Type alignment: emit `_Alignas(N)` on the first member. Never emit
   `__attribute__((aligned(N)))` on a struct or typedef. tcc ignores that
   too.
3. Static alignment to 4 MiB works. Heap alignment to 4 MiB works
   through `posix_memalign` and through manual over-align. The dev
   allocator in 3.7 can meet the ts-rust 4 MiB rule. Declare
   `aligned_alloc` and `posix_memalign` explicitly in runtime C (K3-18).
4. Stack locals: `_Alignas` is trustworthy only to 16 bytes. A Rust local
   or field that needs more must not take its address from a plain local
   declaration. Options, in order: put the value in a struct whose first
   member carries `_Alignas` and keep the struct at a known location
   (this still bottoms out at stack alignment for locals over 16), or
   allocate over-aligned storage and align the pointer up by hand. u128
   at 16 bytes is safe. Types with alignment 32 and above on the stack
   need the manual path, or a heap box. Wave 2 should pick one rule and
   test it. ts-rust alignments above 16 on stack values should be rare,
   but "rare" is not "none".

### shims (ARCH 3.7, K6): cdylib wall for dev, system linker as the static option

Recommended dev link path:

1. Build the Rust shim crate as a **cdylib** (`liblustshim.so`), with the
   system toolchain that rustc already uses. It owns all Rust unwinding,
   Rust TLS, and Rust threads. dyld and ld.so handle its `.eh_frame`,
   `.init_array`, and TLS. Every semantic test passes in this shape.
2. In the daemon layout, libbase (tcc-built) links the cdylib by `-l`
   with an rpath, exactly as `link_with_base` links libbase today.
   Probe K6-11 proves this shape works through libtcc. Overlay exes
   then never name the shim at all, and overlay relinks stay at the
   current speed. The cost of a shim-aware relink stays at 4 to 6 ms.
3. For a standalone dev binary (no daemon), libtcc plus cdylib also
   passes and is the fastest correct path (3.5 ms). Ship the cdylib
   next to the binary, or load it by dlopen with an explicit path (K6-6
   also passes). dlopen is the better default for standalone runs
   because a missing shim then gives one clear error at a known point.
4. If a static dev binary is ever required, the only proven path is:
   tcc objects built with `-fno-asynchronous-unwind-tables`, final link
   by the system linker (`cc`), staticlib input. Budget about 195 ms at
   200-export scale. That cost is why this is not the dev default.
5. Never final-link a Rust staticlib with the tcc linker, in the CLI or
   through libtcc, in any layout (exe, libbase, overlay). This includes
   shims that promise never to panic: Rust thread spawn breaks without
   any user panic (K6-3).
6. Abort-profile shims (`panic = "abort"`) do not rescue path 5. K6-3
   has no panic to abort on and still dies.
7. Allocator rule from 3.7 stands as written. Cross-free happens not to
   crash with the system allocator (K6-13), but the API must keep one
   owner per buffer because the shim allocator is a build-time choice
   (jemalloc in the ts-rust port). Copy-out into caller buffers and
   shim-freed shim buffers are both proven in `k6/main.c`.
8. Callback rule from 3.7 is proven: the shim stops calling back after
   the first non-zero status (calls equal 4, sum equal 6 in
   `shim_callback_stop_on_status`).

### eh_frame hygiene (new, for `src/tcc.rs`)

Pass `-fno-asynchronous-unwind-tables` in every tcc compile lust runs.
Two reasons. First, tcc FDEs overlap in any unit with more than one
function, which breaks any later system-linker step and poisons any
debugger or profiler that walks lust frames. Second, lust C frames never
unwind, so the bytes buy nothing. Debug builds keep `-g` line info. If a
stack walk through lust frames is ever required for diagnostics, that is
a separate feature with its own table, not `.eh_frame` from tcc.

## macOS arm64: what the tcc source says, and what to run there

No Mac result exists yet. `spikes/macos/PROBES.md` is the full probe
list (M0 to M13) and `spikes/macos/run-macos.sh` is the runnable script.
The summary of the pinned source (tinycc 43c7708b, read on Linux):

| Feature | tcc source says | Source pointer |
|---|---|---|
| arm64 Mach-O target exists | `configure` sets `TCC_TARGET_MACHO` for Darwin. `arm64-gen.c`, `arm64-link.c`, and `tccmacho.c` build the target. | `configure` line 673 |
| Atomics 1/2/4/8 bytes | Size gate is target-generic (8 max). The libtcc1 helpers exist for aarch64 and `is_lock_free` returns true at size 8 on aarch64. Expect the same pass as Linux. | `tccgen.c` atomic size check; `lib/stdatomic.c`; `lib/atomic.S` aarch64 block at line 1502 |
| 128-bit atomics, `__int128` | Same rejections as Linux. `__int128_t` is only a fake 16-byte struct so headers parse. | `include/tccdefs.h` lines 182-186 |
| Spin hint | No pause or yield builtin anywhere in the source. Use a compiler barrier, and test `__asm__ volatile("yield")` on the machine. | grep for `ia32_pause` and `yield` across the tree: no builtin |
| TLS on Mach-O | The arm64 backend emits ELF local-exec TLS (`mrs x, tpidr_el0` plus `R_AARCH64_TLSLE_ADD_TPREL_*`) for every non-PE target. macOS TLS uses TLV descriptors, not TPREL. The Mach-O writer has no TLV or `__thread_data` handling. Its only `STT_TLS` case maps the symbol to a plain section symbol. Expect `_Thread_local` to act as plain shared data or to fail the link. Either way the pthread-key rule is expected to stand. Measure it (probe M8). | `arm64-gen.c` lines 471-487; `arm64-link.c` lines 46-47, 372-381; `tccmacho.c` line 1061 |
| Alignment | `MAX_ALIGN` is 16 on arm64, same as x86_64. Attribute handling is shared code, so expect the same ignores as Linux. Mach-O section alignment tops out at 2^15 in the format, so a 4 MiB-aligned section is not representable. The heap path must carry the 4 MiB case. | `arm64-gen.c` line 39; `tccgen.c` lines 4024-4040; `tccmacho.c` line 1850 |
| Unwind info on Mach-O | Unwind tables are forced off for non-ELF output. There is no compact-unwind writing for foreign objects. The Linux staticlib failure mode should be at least as bad on Mach-O. | `tccelf.c` lines 94-96 |

Exact commands for the Mac run are in `spikes/macos/run-macos.sh`. The
script has never run on a Mac. It needs three local decisions when it
first runs: the right tcc flag for a Mach-O shared library (it tries
`-shared`, then `-Wl,-dynamiclib`), whether `-Wl,-rpath` from tcc
produces `LC_RPATH`, and the system frameworks a Rust staticlib needs on
the `cc` line. Those are marked in the script. Do not mark tier-one
macOS support until probes M2, M7, M8, M10, and M11 have log lines.

## Things this spike did not prove

1. Weak CAS failure rates and ordering strength beyond the functional
   stress test. tcc lowers C11 atomics to libtcc1 helpers that use
   seq_cst CAS loops for the fetch ops (`lib/stdatomic.c`), so orderings
   are stronger than written, never weaker. That is safe for rule zero.
   It can cost speed on relaxed fetch ops under contention. No
   contention benchmark ran here.
2. Atomics misaligned by construction (an `_Atomic` at a non-natural
   address). Rust never does this legally. The helpers assume natural
   alignment, as does C.
3. `u128.h` under the in-memory libtcc mode. The functions are plain C
   and the mode runs the atomics probe, so the risk is low, but no probe
   ran the vector test through `tcc_relocate`.
4. The cdylib wall with a jemalloc global allocator inside the shim (the
   ts-rust shape). The probe shim uses the system allocator. The arena
   and 4 MiB rules in 3.7 need their own probe in W-SHIM when the real
   jemalloc shim exists.
5. Fork safety of `run_in_memory` with pthread-key TLS destructors in
   the child. Probes run destructors on thread exit in a normal process
   only.
6. Any macOS number at all.
