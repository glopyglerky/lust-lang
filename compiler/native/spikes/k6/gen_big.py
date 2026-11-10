#!/usr/bin/env python3
"""K6: generate a realistic-size shim (~200 exports) for link timing.

Writes shim_big.rs, main_big.c, shim_big.h into the output dir (argv[1]).
Each export does a small mix of work so the object is not trivial.
"""
import pathlib
import sys

N = 200
out = pathlib.Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)

rs = ["use std::panic::catch_unwind;\n"]
for i in range(N):
    rs.append(
        f"""
#[unsafe(no_mangle)]
pub extern "C" fn big_fn_{i}(x: i64) -> i64 {{
    let mut acc = x.wrapping_mul({i + 3});
    for k in 0..8 {{
        acc = acc.wrapping_add((acc >> (k + 1)) ^ {i * 7 + 1});
    }}
    if x == i64::MIN + {i} {{
        let _ = catch_unwind(|| panic!("big {i}"));
    }}
    acc
}}
"""
    )
rs.append(
    """
#[unsafe(no_mangle)]
pub extern "C" fn big_catch() -> i32 {
    std::panic::set_hook(Box::new(|_| {}));
    i32::from(catch_unwind(|| panic!("big catch")).is_err())
}
"""
)
(out / "shim_big.rs").write_text("".join(rs))

hdr = ["#ifndef SHIM_BIG_H\n#define SHIM_BIG_H\n"]
hdr += [f"long big_fn_{i}(long x);\n" for i in range(N)]
hdr += ["int big_catch(void);\n#endif\n"]
(out / "shim_big.h").write_text("".join(hdr))

c = ['#include <stdio.h>\n#include "shim_big.h"\nint main(void) {\n    long s = 0;\n']
for i in range(N):
    c.append(f"    s += big_fn_{i}({i + 1});\n")
c.append(
    '    int caught = big_catch();\n'
    '    printf("INFO big sum=%ld caught=%d\\n", s, caught);\n'
    '    int ok = caught == 1 && s != 0;\n'
    '    printf("RESULT %s fails=%d\\n", ok ? "PASS" : "FAIL", ok ? 0 : 1);\n'
    "    return ok ? 0 : 1;\n}\n"
)
(out / "main_big.c").write_text("".join(c))
print(f"generated {N} exports in {out}")
