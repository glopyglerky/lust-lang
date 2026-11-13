// The generic reuse oracle (`LUST_CHECK_GENREUSE=1`). Generic reuse keeps
// results across requests of a session (driver/gen_reuse.rs and the generic
// reuse walk in compile.rs): instance and closure bodies, with their C, and
// the stored template, closure template and discovery entries. A const fn
// edit once moved const values under all of them with no oracle noticing
// (fuzz small2 seed 11, 2026-10-19), so this one proves each kept result:
// - reads: every instance or closure body the walk keeps, and every stored
//   entry kept because its environment stands, must have stored reads that
//   are clean against this program (whatever shortcut kept it; `Checked` reads
//   compare by no fingerprint and are left to the C compare);
// - C: in a release C build (content-addressed names), every concrete body the
//   request does not check again is checked, lowered and generated again
//   without reuse, and its C must equal the C the session keeps for it.
// A difference fails as the patch oracle does (`item_oracle::fail`: a line
// in `LUST_ITEMPATCH_LOG`, on stderr, and a `Failure` panic).
use crate::check::collect::Program;
use crate::def::LocalDefId;
use crate::driver::graph::{Read, ReadKey, ReadKeys, Reads};
use crate::intern::FxMap;

/// True when `LUST_CHECK_GENREUSE` is set (and not `0`).
pub(crate) fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("LUST_CHECK_GENREUSE").is_ok_and(|v| !v.is_empty() && v != "0"))
}

/// Fails when a kept result (`what`, of fn `f`) has a stored read that answers
/// otherwise in this program, naming the first. `Checked` reads have no
/// fingerprint to compare (every replay counts them moved); the walks follow
/// them by the callee's slot, and the C compare below covers them.
///
/// A key whose answer can move while mono runs (an `Answer` key, or one whose
/// answer read state mono still writes: `Program::read_fingerprint_stable`) is
/// answered fresh, as the check that stored it answered it at its turn: `memo`
/// holds the answer of the key's first use in this collect, which can come
/// from before mono (an auto trait answer the instances decide later reads as
/// undetermined there).
pub(super) fn kept_reads(what: &str, p: &Program, f: LocalDefId, reads: &Reads, keys: &ReadKeys, memo: &mut FxMap<ReadKey, Read>) {
    let mut now = |k: &ReadKey| -> Read {
        if !matches!(k, ReadKey::Answer(..))
            && let Some(&r) = memo.get(k)
        {
            return r;
        }
        let (r, stable) = p.read_fingerprint_stable(k);
        if stable && !matches!(k, ReadKey::Answer(..)) {
            memo.insert(*k, r);
        }
        r
    };
    let mut moved: Vec<(ReadKey, Read, Read)> = Vec::new();
    for (k, stored) in reads.iter(keys) {
        if matches!(k, ReadKey::Checked(_)) {
            continue;
        }
        let r = now(k);
        if r != stored {
            moved.push((*k, stored, r));
        }
    }
    if let Some(&(k, stored, r)) = moved.first() {
        super::item_oracle::fail(
            &format!("generic reuse kept {what} whose reads moved"),
            "-",
            &format!("{}: {} reads, first {k:?}: stored {stored:?}, now {r:?}", fn_name(p, f), moved.len()),
        );
    }
}

/// Compares the C a build keeps for the bodies it did not check again
/// (`kept`, by local fn index) with the C made again without reuse
/// (`fresh`: the code, LLVM IR and helper C, or the fresh check's error).
pub(super) fn compare_bodies(
    p: &Program,
    path: &str,
    kept: &[usize],
    fresh: Vec<Result<[String; 3], String>>,
    stored: impl Fn(usize) -> [String; 3],
) {
    for (&i, fresh) in kept.iter().zip(fresh) {
        let f = LocalDefId(i as u32);
        let what = |w: &str| format!("{w} ({:?})", p.gx.fn_class.get(i));
        match fresh {
            Err(e) => super::item_oracle::fail(
                &what("generic reuse kept a body that a fresh check rejects"),
                path,
                &format!("{}: {e}", fn_name(p, f)),
            ),
            Ok(fresh) => {
                let kept = stored(i);
                if let Some(k) = (0..3).find(|&k| kept[k] != fresh[k]) {
                    let (a, b) = (kept[k].lines().collect::<Vec<_>>(), fresh[k].lines().collect::<Vec<_>>());
                    let n = (0..a.len().max(b.len())).find(|&n| a.get(n) != b.get(n)).unwrap_or(0);
                    let detail = format!(
                        "{}: line {}: kept `{}`, fresh `{}`",
                        fn_name(p, f),
                        n + 1,
                        a.get(n).unwrap_or(&""),
                        b.get(n).unwrap_or(&"")
                    );
                    super::item_oracle::fail(&what("generic reuse kept C that differs from a fresh build"), path, &detail);
                }
            }
        }
    }
}

fn fn_name(p: &Program, f: LocalDefId) -> String {
    p.display_name(p.fn_def(f).name).to_string()
}
