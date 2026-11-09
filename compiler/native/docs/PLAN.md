# lust: what we do next

Draft 4. The north star is ts-rust, and the main decisions are made.

**North star:** lust compiles a project like [ts-rust](https://github.com/pingdotgg/ts-rust) (786k lines, a Rust port of the TypeScript 7 compiler) **100x faster than rustc** in the dev loop, with **no speed loss in the shipped binary**. Code and syntax changes are OK if an agent has a clear path to port.

Sources: 8 experiments with cross-model checks, 2 review rounds (Muse, sol), and 3 ts-rust studies: a feature inventory by a real parser (Opus), a rustc baseline on ts-rust (Muse), and a port design (sol). All numbers are wall time on a 24-core Linux box.

## The short version

1. **The shipped binary is free.** Every lust program is valid Rust (rule zero). So a release build hands the same source to Cargo with ts-rust's own recipe (fat LTO, PGO, BOLT, jemalloc). The binary is the same one that ships today. lust owns only the dev loop.
2. **The 100x is in the edit loop.** rustc needs 12 s to check and 21 s to build after a one-line edit in ts-rust. 100x is 121 ms and 206 ms. lust's daemon cost grows with the edit, not the crate, so this is reachable.
3. **Fresh cold 100x is hard.** The targets are 290 ms check and 430 ms build for 786k lines. lust is about 6x and 9x too slow per line today, before it has traits and generics.
4. **The language gap is big.** ts-rust needs 52k to 80k more lines of lust (Opus and sol agree). The hard parts are closures, traits, `dyn`, `Rc`/`RefCell`, lifetimes on structs, real threads, and panic recovery.
5. **Progress is flat, then steep.** Almost every ts-rust file uses closures, `Rc`/`RefCell`, `?`, tuples, and glob imports together. So almost no file compiles until about 6 big features land. We gate on targeted spikes and vertical slices, not on "percent of ts-rust".

## The targets

rustc's best setup on ts-rust (`-Z threads=8 -C debuginfo=0` for cold, incremental for edits):

| Operation | rustc | 100x target | lust (estimate at 786k) | Gap |
|---|---|---|---|---|
| Edit loop: body edit, check | 12.1 s | 121 ms | about 7 ms at 31k; grows with the edit | Likely OK |
| Edit loop: body edit, build | 20.6 s | 206 ms | about 14 ms at 31k; grows with the edit | Likely OK |
| Edit loop: type change, build | 60.2 s | 602 ms | full rebuild, about 3.8 s | Needs finer invalidation |
| Edit loop: a mistake, check | 8.8 s | 88 ms | grows with the edit | Likely OK |
| Cold check, crate only | 29.0 s | 290 ms | about 1.75 s | 6x per line |
| Cold debug build, crate only | 43.0 s | 430 ms | about 3.8 s | 9x per line |
| Release build | 155 s plain, 708 s fat LTO (one bin) | n/a | Cargo builds it | Same binary |

Where rustc spends a cold check: type checking 35%, name resolution 24%, borrow checking 12%, macro expansion 10%. In a cold build no pass dominates. Cranelift made rustc slower on ts-rust (53.8 s against 43.0 s), because the front end is the bigger cost.

**The dev binary's speed also matters.** ts-rust's release binary checks a test project in 0.37 s. A rustc debug binary takes 6.6 s (18x). A tcc binary would take about 11 s (estimate). An agent that runs ts-rust's tests after each edit waits on that. So a faster dev binary is part of the 100x story for this project.

## Decisions

1. **Cache:** local and ignored by git, like Go's build cache. A shared cache is a later topic.
2. **Compile on save:** parked.
3. **`unsafe`:** yes, opt-in. ts-rust forbids `unsafe` in its own code, so the port needs it only for C shims.
4. **Release goes through Cargo** with the project's own recipe. lust stays a strict Rust subset. No lust-only syntax for now.
5. **Threads and panic recovery are in the plan** (milestone 7). ts-rust on one thread is 2.8x to 3.2x slower, and it catches panics in 59 places.
6. **Lifetimes are in the plan** (milestone 6). The "no lifetime syntax" and "no references in struct fields" rules are dropped. lust uses a restricted Rust lifetime model, with elision as the default.
7. **Macros: native `macro_rules!`**, with cached expansion (milestone 4). Why not the one-time codemod:
   - It would add up to 29k lines of expanded code to ts-rust, and 3,023 call sites would turn into boilerplate. People and agents would then maintain the expanded code.
   - Every real Rust project uses `macro_rules!`. Native support makes the next port easy too.
   - ts-rust's macros use only simple fragments (`expr`, `ident`, `ty`, `tt`, `literal`, `meta`), so the first version can be small. Proc macros stay out.
   - The codemod stays as a fallback for a macro that lust cannot expand yet.
8. **First port scope: the core CLI** (402 files, 388k lines). LSP, watch, API, and the Effect plugin come after.

## What ts-rust needs

From Opus's inventory (571k handwritten lines, 19,492 functions), ranked by functions touched times difficulty:

| # | Feature | Functions | In the old plan? |
|---|---|---|---|
| 1 | General closures: stored, `Rc<dyn Fn>`, passed to user fns | 3,733 | Only fused adapters |
| 2 | Traits and generics | 2,742 | Yes |
| 3 | `Rc`, `Weak`, `RefCell`, `Cell`, `OnceCell` | 2,608 | No |
| 4 | Iterators (3,516 chains) | 2,303 | Yes |
| 5 | Arrays and slices | 2,674 | Yes |
| 6 | Lifetimes in user types (168 types) | 1,051 | Rejected |
| 7 | `dyn Trait` (94 user traits, `dyn Fn`, `dyn Any`) | 1,086 | No |
| 8 | `Result`, `?`, let-else | 2,135 | Yes |
| 9 | Threads and sync | 845 | Rejected |
| 10 | Dependencies (FxHashMap, IndexMap, SmallVec, syscalls) | 975 | No |

Good news:

- **The arena problem is already solved.** ts-rust's AST, symbols, and types use integer handles (`Node(u64)`, `TypeId`, and 48 more). Long-lived data is leaked and held as `&'static`.
- **Lifetimes are mostly cheap.** `'static` (1,228 uses) and `'_` (1,364 uses) need no region inference. Supporting them costs 1k to 2k lines.
- **Few generics.** 591 generic functions (3%). No GATs, no proc macros, no unions, 6 blanket impls.

**Dependencies.** None fit the lust subset as source (unsafe, SIMD, asm). The plan:

- **Built into lust:** Fx `HashMap`/`HashSet`, `IndexMap`/`IndexSet`, `SmallVec`, `memchr`, xxh3, ryu-js, a leak arena, and a small BigInt. 4k to 7k lines. Iteration order must match the real crates where output depends on it.
- **C shims, built once by rustc and cached:** rustix, nix, signal-hook, jemalloc, zlib, and local time.
- **Release:** the real crates, through Cargo.

## Milestones

Each milestone has a gate. Line counts are lust compiler and runtime only.

| # | Milestone | Gate | Lines |
|---|---|---|---|
| 1 | **Correct under stress.** Fix the 8 known bugs (false daemon errors, a crash on `/*`, a hang on `(`, MIN literals, reserved words, 2 borrowck gaps, `libbase` reuse). Add the fresh-session guard. Make the random-edit fuzzer a CI gate. Lock stable, module-qualified IDs. Start Mac CI. | 100,000 random edits with no disagreement | 1k to 3k |
| 2 | **Instant and honest.** Autostart the daemon. Exit without cleanup. Bigger cold units. Hybrid glue. In-memory small runs. A fixed whole-request disk cache. `lust build --release` and `--profile` hand off to Cargo. Add ts-rust's edit loop as a benchmark. | Agents get warm speed with no setup. The release binary is identical to Cargo's. | 2k to 4k |
| 3 | **Agent kit.** Versioned JSON diagnostics with byte edits. `lust test` with a process per test. Narrow `--fix` (no auto clones). `lust explain` and `lust capabilities`. `lust port scan`: a full Rust parser that reports every gap in a crate, grouped by root cause. | `lust port scan` gives a complete gap report for ts-rust | 3k to 5k |
| 4 | **The Rust core.** Modules with globs and re-exports, `cfg`, `const` and `static`, aliases, tuples, arrays, slices, `Box`, `Result`/`?`/let-else, labels, struct update, `if let` chains, u128, f64, format specs, `Drop`, more derives, std `mem`/`env`/`fs`/`io`/`path`/`time`, about 25 std macros, and native `macro_rules!` (fragments, repetition, `$crate`, hygiene, cached expansion with a size cap). | The `gostring` kernel (tests/kernels) runs with the same bytes as rustc. All 70 ts-rust macro names pass `scripts/test-macros.sh`. The full scanner and JSON kernels need named lifetimes and traits, so they are the milestone 6 gate. | 13k to 20k |
| 5 | **Traits, closures, `dyn`, `Rc`.** User traits and std trait impls, mono generics, associated types and consts, a deterministic impl index, general closures (stored, `Rc<dyn Fn>`, higher-ranked `Fn(&T)`), fused iterators with an unfused fallback, vtables, `dyn Any`, `Rc`/`RefCell`/`Cell`/`OnceCell`/`LazyLock`/`Cow`, and the built-in dependency layer. | **Compatibility spike:** `NodeVisitor`, `ArenaBox`, one blanket impl, `go_flags!` output, and a stored callback compile and match rustc | 25k to 38k |
| 6 | **Lifetimes.** First `'static` and `'_` (takes ts-rust from 17% to 33% of lines). Then a restricted Rust lifetime model: params on structs, enums, impls, and aliases, variance, outlives, rank-one binders, and drop check. | `Parser<'a>`, `JsonDecoder<'a>`, and `RuleContext<'a, 'c>` compile. Escape and aliasing tests reject, like rustc. The scanner and JSON kernels (tests/kernels) run with the same bytes as rustc. | 5k to 9k |
| 7 | **Threads and panics.** `Send`/`Sync`, `Arc`, `Mutex`, `RwLock`, `Condvar`, atomics, mpsc, `thread::scope`, `thread_local!`. Panic recovery with drops on unwind (spike explicit propagation in the C backend first). C shims for syscalls and jemalloc. | Parallel bind and check run. A recovered panic matches rustc. | 9k to 15k |
| 8 | **Port ts-rust, core CLI first.** `lust port apply` with checked rewrites only. Scope: the 402 core files (checker, binder, parser, emitter, program, transformers, and helpers). Then 8b: LSP, watch, API, the Effect plugin, and a small async state machine for ATA. | 8a: the CLI's baseline tests pass with the same output. 8b: all of ts-rust's baseline tests pass. The release binary comes from Cargo, at the same speed. | 3k to 6k |
| 9 | **100x at ts-rust scale.** Finer invalidation for type and impl edits. An instance cache in the daemon. Front-end speed work for cold check. A backend bake-off (tcc, Cranelift, copy-and-patch) for build speed and dev-binary speed. | Measured 100x on the ts-rust edit loop. Cold check and build reported as separate rows. | 4k to 8k |

**Total:** about 63k to 105k lines over today's 11k, plus 15k to 30k lines of tests. The two independent estimates (Opus 52k to 80k, sol 54k to 99k) agree.

Order notes:

- Milestones 1 to 3 do not change. They pay off on every project, and they build the tools (`lust port scan`, the fuzzer) that make the rest safe.
- Run sol's **compatibility spike** early in milestone 5, before committing to the full range. It tests each big design choice once: a visitor with stored callbacks, `ArenaBox` with `Deref`, a blanket impl, a scoped thread, a recovered panic, one C shim, and one borrowed async task.
- The core CLI scope follows the same curve: 17% of its lines compile after milestone 5, 40% after `'static` and `'_`, 68% after full lifetimes, and 84% after threads.
- `'static` and `'_` are the cheapest big win in the plan: 1k to 2k lines for +16% of ts-rust's lines. Do them with or before `dyn`.
- Measure ts-rust-scale speed from milestone 2, so every feature lands with its cost visible.

## 100x, honestly

- **Edit loop:** the main claim. rustc's floor on ts-rust is 12 s per check, because the crate is one 786k-line unit. lust's daemon re-checks only what an edit touches. Body edits, new functions, and signature edits should land well under 121 ms. Type changes need finer invalidation (rustc pays 60 s there).
- **Cold check and build:** their own rows, never blended. They need about 6x and 9x faster per line, and traits and generics will add cost. This is a research target. The front end matters most.
- **Cached cold:** a fresh process that loads this project's artifacts from disk. Its own row.
- **The shipped binary:** Cargo builds it, so it is the same binary.

## What we will not do

- Lust-only syntax (for now). Release must stay plain Cargo.
- Codemods that change data structures (`Rc<str>` for `&str`, one thread instead of many, clones). They slow the shipped binary.
- The interpreter (D) and dictionary passing for generics (G).
- Compile on save (parked).
- Proc macros, GATs, specialization, and general async executors.
- A one-time macro codemod as the main path (it stays a fallback).
