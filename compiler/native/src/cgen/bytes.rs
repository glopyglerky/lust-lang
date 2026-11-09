// Integer byte conversions in codegen (TRIAGE R26): `to_le_bytes`,
// `to_be_bytes`, `to_ne_bytes`, and the `from_*_bytes` constructors copy
// the value's bytes (the C target is little endian, so `ne` is `le`),
// and the big endian forms then reverse them. A 128-bit integer's
// `lust_u128`/`lust_i128` keeps its low word first, so the copy holds.
use super::*;

impl FnGen<'_, '_> {
    /// One byte conversion builtin with its destination type, or None
    /// for every other builtin.
    pub(super) fn bytes_builtin(&mut self, b: Builtin, ops: &[Operand], dest: &CTy, loc: &str, post: &mut Vec<String>) -> Option<String> {
        use Builtin as B;
        let big = match b {
            B::IntToLeBytes | B::IntToNeBytes | B::IntFromLeBytes | B::IntFromNeBytes => false,
            B::IntToBeBytes | B::IntFromBeBytes => true,
            _ => return None,
        };
        let (v, t) = self.operand(&ops[0], loc, post);
        let n = match (&t, dest) {
            (CTy::Array(_, n), _) | (_, CTy::Array(_, n)) => *n,
            _ => unreachable!("byte conversion without a byte array"),
        };
        let (s, d) = (self.tmp(), self.tmp());
        let _ = writeln!(self.out, "    {} {s} = {v}; {} {d}; lust_memcpy(&{d}, &{s}, {n});", ctype(&t), ctype(dest));
        if big {
            let (p, i, c) = (self.tmp(), self.tmp(), self.tmp());
            let _ = writeln!(
                self.out,
                "    uint8_t *{p} = (uint8_t *)&{d}; for (uint32_t {i} = 0; {i} < {n} / 2; {i}++) {{ uint8_t {c} = {p}[{i}]; {p}[{i}] = {p}[{n} - 1 - {i}]; {p}[{n} - 1 - {i}] = {c}; }}"
            );
        }
        Some(d)
    }
}
