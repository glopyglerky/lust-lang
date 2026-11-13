// The runtime's small fns as inlinable IR for LLVM backend units (`LUST_LL_RTINL`).
//
// The runtime unit compiles to LLVM IR once (cc.rs), and `split` makes two modules of it:
// - the runtime object's IR: every internal symbol (static fns, static data, fn-local
//   statics) becomes hidden and external under the name `lust.rt.<name>`, so code inlined
//   into other units can still reach it;
// - a copy for the LLVM units: each fn of at most `max` instructions as
//   `available_externally` (clang may inline it, never emits it), every other fn as a
//   declaration, constants as `available_externally` (loads from tables fold), mutable
//   globals as declarations, private constants (string literals) as they are, and no
//   `llvm.*` arrays or module asm (those belong to the runtime object only).
// Each LLVM unit links the copy as builtin bitcode (only what it names), so a hash probe
// or an Arc count update inlines per unit as it would under LTO.
use std::collections::HashSet;

/// The two IR texts `split` makes.
pub struct RtIr {
    /// The runtime object's IR.
    pub obj: String,
    /// The copy for LLVM units.
    pub inl: String,
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'$' | b'.' | b'_')
}

/// The global name a top-level line defines (`define ... @f(` or `@g = ...`).
fn defined_name(line: &str) -> Option<&str> {
    let at = if line.starts_with("define ") { line.find(" @")? + 1 } else if line.starts_with('@') { 0 } else { return None };
    let rest = &line[at + 1..];
    let end = rest.bytes().position(|b| !is_ident(b)).unwrap_or(rest.len());
    Some(&rest[..end])
}

/// `line` with each `@name` of `names` renamed to `@lust.rt.name` (not inside `"..."`: IR
/// strings escape their quotes, so a quote always opens or closes one).
fn rename(line: &str, names: &HashSet<&str>) -> String {
    let b = line.as_bytes();
    let mut out = String::with_capacity(line.len() + 16);
    let (mut i, mut from) = (0, 0);
    while i < b.len() {
        match b[i] {
            b'"' => i += line[i + 1..].find('"').map_or(b.len() - i, |e| e + 2),
            b'@' => {
                let mut end = i + 1;
                while end < b.len() && is_ident(b[end]) {
                    end += 1;
                }
                if names.contains(&line[i + 1..end]) {
                    out.push_str(&line[from..=i]);
                    out.push_str("lust.rt.");
                    from = i + 1;
                }
                i = end;
            }
            _ => i += 1,
        }
    }
    out.push_str(&line[from..]);
    out
}

/// The type that starts `s` (one token, or a bracketed aggregate), and the rest.
fn split_type(s: &str) -> (&str, &str) {
    let b = s.as_bytes();
    let mut depth = 0i32;
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'[' | b'{' | b'<' | b'(' => depth += 1,
            b']' | b'}' | b'>' | b')' => {
                depth -= 1;
                if depth == 0 {
                    return (&s[..=i], &s[i + 1..]);
                }
            }
            b' ' | b',' if depth == 0 => return (&s[..i], &s[i..]),
            _ => {}
        }
    }
    (s, "")
}

/// The copy-side line of a global: an `available_externally` constant, a declaration of a
/// mutable global, a private constant as it is, or None (an `llvm.*` array).
fn inl_global(line: &str, promoted: bool) -> Option<String> {
    let (lhs, rhs) = line.split_once(" = ")?;
    if lhs.starts_with("@llvm.") {
        return None;
    }
    let words: Vec<&str> = rhs.split(' ').collect();
    let kind = words.iter().position(|w| *w == "global" || *w == "constant")?;
    // Private data (string literals) and declarations stay as they are.
    if words[..kind].iter().any(|w| matches!(*w, "private" | "external" | "extern_weak")) {
        return Some(line.to_string());
    }
    let quals: Vec<&str> = words[..kind].iter().copied().filter(|w| !matches!(*w, "internal" | "dso_local" | "hidden") && !interposable(w)).collect();
    let vis = if promoted || words[..kind].contains(&"hidden") { "hidden " } else { "" };
    let quals = quals.iter().map(|w| format!("{w} ")).collect::<String>();
    // A constant another definition may replace (weak, common) is only declared.
    if words[kind] == "constant" && !words[..kind].iter().any(|w| interposable(w)) {
        return Some(format!("{lhs} = available_externally {vis}{quals}{}", words[kind..].join(" ")));
    }
    let after = words[kind + 1..].join(" ");
    let (ty, rest) = split_type(&after);
    let align = rest.find(", align ").map_or("", |a| {
        let r = &rest[a..];
        &r[..r[8..].find(|c: char| !c.is_ascii_digit()).map_or(r.len(), |e| e + 8)]
    });
    Some(format!("{lhs} = external {vis}{quals}{} {ty}{align}", words[kind]))
}

/// A linkage another definition may replace at link time (the runtime's weak fallbacks):
/// its body must not be inlined anywhere.
fn interposable(word: &str) -> bool {
    matches!(word, "weak" | "weak_odr" | "linkonce" | "linkonce_odr" | "common")
}

/// The declaration of the fn whose `define` line is `line` (already renamed): the header
/// without its linkage, body, personality, section or metadata.
fn declaration(line: &str, promoted: bool) -> String {
    let head = line.trim_end().strip_suffix('{').unwrap_or(line).trim_end();
    let head = [" personality ", " section ", " comdat", " prefix ", " prologue ", " !"].iter().fold(head, |h, cut| h.find(cut).map_or(h, |at| &h[..at]));
    let rest = head.strip_prefix("define ").unwrap_or(head);
    let rest = match rest.split_once(' ') {
        Some(("internal", r)) if promoted => format!("hidden {r}"),
        Some((w, r)) if w == "internal" || w == "external" || interposable(w) => r.to_string(),
        _ => rest.to_string(),
    };
    format!("declare {rest}")
}

/// The metadata lines of the copy without the module's ProfileSummary: its flag in
/// `!llvm.module.flags` and the nodes only it reaches. A PGO build's summary holds counts of
/// the runtime unit's whole profile slice, which moves with every merge (records that other
/// units define change hands) while the fn bodies stay the same; each unit replaces the
/// summary with its own profile's anyway. Without it the copy, and so every LLVM unit's
/// key, stays the same across merges.
fn drop_profile_summary(lines: &mut Vec<String>) {
    let node = |l: &str| -> Option<usize> { l.strip_prefix('!')?.split_once(" = ")?.0.parse().ok() };
    let refs = |l: &str| -> Vec<usize> {
        let mut out = Vec::new();
        let b = l.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'"' {
                i += l[i + 1..].find('"').map_or(b.len() - i, |e| e + 2);
                continue;
            }
            if b[i] == b'!' && b.get(i + 1).is_some_and(u8::is_ascii_digit) {
                let end = (i + 1..b.len()).find(|&j| !b[j].is_ascii_digit()).unwrap_or(b.len());
                out.extend(l[i + 1..end].parse::<usize>().ok());
                i = end;
                continue;
            }
            i += 1;
        }
        out
    };
    let Some((flag, root)) = lines.iter().find_map(|l| {
        let n = node(l)?;
        l.contains(", !\"ProfileSummary\", !").then(|| refs(l.split_once(" = ")?.1).last().copied().map(|r| (n, r)))?
    }) else {
        return;
    };
    let at: std::collections::HashMap<usize, usize> = lines.iter().enumerate().filter_map(|(i, l)| node(l).map(|n| (n, i))).collect();
    let mut sub: HashSet<usize> = HashSet::from([flag]);
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if sub.insert(n) && let Some(&i) = at.get(&n) {
            stack.extend(refs(lines[i].split_once(" = ").map_or("", |x| x.1)));
        }
    }
    // Keep a node something outside the summary also names (another module flag included).
    let outside: HashSet<usize> = lines
        .iter()
        .filter(|l| !node(l).is_some_and(|n| sub.contains(&n)))
        .flat_map(|l| refs(l.split_once(" = ").filter(|_| l.starts_with('!')).map_or(l.as_str(), |x| x.1)))
        .filter(|&n| n != flag)
        .collect();
    sub.retain(|n| !outside.contains(n));
    lines.retain(|l| !node(l).is_some_and(|n| sub.contains(&n)));
    for l in lines.iter_mut().filter(|l| l.starts_with("!llvm.module.flags = !{")) {
        let kept: Vec<String> = refs(l).into_iter().filter(|n| !sub.contains(n)).map(|n| format!("!{n}")).collect();
        *l = format!("!llvm.module.flags = !{{{}}}", kept.join(", "));
    }
}

/// Splits the runtime unit's IR (clang `-S -emit-llvm`) into the runtime object's IR and
/// the copy for LLVM units, whose fns of at most `max` instructions are inlinable.
pub fn split(ir: &str, max: usize) -> RtIr {
    let lines: Vec<&str> = ir.lines().collect();
    let internal: HashSet<&str> = lines
        .iter()
        .filter(|l| l.starts_with("define internal ") || (l.starts_with('@') && l.contains(" = internal ")))
        .filter_map(|l| defined_name(l))
        .collect();
    let mut obj = String::with_capacity(ir.len() + ir.len() / 16);
    let mut inl = String::with_capacity(ir.len());
    let mut i = 0;
    while i < lines.len() {
        let line = rename(lines[i], &internal);
        let promoted = defined_name(lines[i]).is_some_and(|n| internal.contains(n));
        if line.starts_with("define ") {
            // The whole body: up to the closing brace at column 0.
            let end = (i..lines.len()).find(|&j| lines[j] == "}").unwrap_or(lines.len() - 1);
            let body: Vec<String> = lines[i + 1..=end].iter().map(|l| rename(l, &internal)).collect();
            let header = if promoted { line.replacen("define internal ", "define hidden ", 1) } else { line.clone() };
            obj.push_str(&header);
            obj.push('\n');
            for l in &body {
                obj.push_str(l);
                obj.push('\n');
            }
            let size = lines[i + 1..end].iter().filter(|l| l.starts_with("  ")).count();
            let replaceable = header.strip_prefix("define ").and_then(|r| r.split(' ').next()).is_some_and(interposable);
            if size <= max && !replaceable {
                let rest = header.strip_prefix("define ").unwrap_or(&header);
                let rest = rest.strip_prefix("external ").unwrap_or(rest);
                inl.push_str("define available_externally ");
                inl.push_str(rest);
                inl.push('\n');
                for l in &body {
                    inl.push_str(l);
                    inl.push('\n');
                }
            } else {
                inl.push_str(&declaration(&line, promoted));
                inl.push('\n');
            }
            i = end + 1;
            continue;
        }
        if line.starts_with('@') && line.contains(" = ") {
            obj.push_str(&if promoted { line.replacen(" = internal ", " = hidden ", 1) } else { line.clone() });
            obj.push('\n');
            if let Some(g) = inl_global(&line, promoted) {
                inl.push_str(&g);
                inl.push('\n');
            }
        } else {
            obj.push_str(&line);
            obj.push('\n');
            if !line.starts_with("module asm ") && !(line.starts_with('$') && line.contains(" = comdat ")) {
                inl.push_str(&line);
                inl.push('\n');
            }
        }
        i += 1;
    }
    let mut inl_lines: Vec<String> = inl.lines().map(str::to_string).collect();
    drop_profile_summary(&mut inl_lines);
    RtIr { obj, inl: inl_lines.join("\n") + "\n" }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IR: &str = r#"@.str = private unnamed_addr constant [2 x i8] c"a\00", align 1
@lust_tls_cur = internal thread_local unnamed_addr global ptr null, align 8
@lust_count = dso_local global { i64, [2 x i32] } zeroinitializer, align 8
@lust_table = dso_local local_unnamed_addr constant [2 x i8] c"xy", align 1
@lust_arc_empty.h = internal global [2 x i64] [i64 1, i64 1], align 16
@llvm.used = appending global [1 x ptr] [ptr @lust_arc_inc], section "llvm.metadata"
@stderr = external local_unnamed_addr constant ptr, align 8
@lust_weak_k = weak dso_local constant i32 7, align 4

define internal fastcc i32 @probe(ptr noundef %0) unnamed_addr #0 {
  %2 = load ptr, ptr @lust_tls_cur, align 8
  %3 = load i32, ptr @lust_arc_empty.h, align 4
  ret i32 %3
}

define dso_local void @lust_arc_inc(ptr noundef %0) local_unnamed_addr #0 {
  %2 = tail call fastcc i32 @probe(ptr %0)
  ret void
}

define weak dso_local void @fallback(ptr noundef %0) local_unnamed_addr #0 {
  ret void
}

define dso_local i64 @big(i64 noundef %0) local_unnamed_addr #0 personality ptr @x {
  %2 = add i64 %0, 1
  %3 = add i64 %2, 1
  %4 = add i64 %3, 1
  ret i64 %4
}

attributes #0 = { nounwind }
"#;

    #[test]
    fn the_copy_drops_the_profile_summary_only() {
        let ir = "define dso_local void @f() #0 !prof !9 {\n  ret void\n}\n!llvm.module.flags = !{!0, !3, !8}\n!0 = !{i32 1, !\"wchar_size\", i32 4}\n!3 = !{i32 1, !\"ProfileSummary\", !4}\n!4 = !{!5, !6}\n!5 = !{!\"TotalCount\", i64 12}\n!6 = !{!\"DetailedSummary\", !7}\n!7 = !{!8}\n!8 = !{i32 7, !\"uwtable\", i32 2}\n!9 = !{!\"function_entry_count\", i64 3}\n";
        let s = split(ir, 10);
        assert!(s.inl.contains("!llvm.module.flags = !{!0, !8}"), "{}", s.inl);
        assert!(!s.inl.contains("ProfileSummary") && !s.inl.contains("TotalCount") && !s.inl.contains("!7 = "), "{}", s.inl);
        assert!(s.inl.contains("!8 = !{i32 7, !\"uwtable\", i32 2}"), "a node named outside stays: {}", s.inl);
        assert!(s.inl.contains("!9 = !{!\"function_entry_count\", i64 3}"));
        assert!(s.obj.contains("ProfileSummary"));
    }

    #[test]
    fn renames_skip_strings() {
        let names: HashSet<&str> = ["probe"].into_iter().collect();
        assert_eq!(rename(r#"@s = c"@probe\22" call @probe(@probe2) @probe"#, &names), r#"@s = c"@probe\22" call @lust.rt.probe(@probe2) @lust.rt.probe"#);
    }

    #[test]
    fn the_object_keeps_every_body_and_exports_the_statics_hidden() {
        let s = split(IR, 2);
        assert!(s.obj.contains("@lust.rt.lust_tls_cur = hidden thread_local unnamed_addr global ptr null, align 8"), "{}", s.obj);
        assert!(s.obj.contains("define hidden fastcc i32 @lust.rt.probe(ptr noundef %0) unnamed_addr #0 {"));
        assert!(s.obj.contains("%2 = tail call fastcc i32 @lust.rt.probe(ptr %0)"));
        assert!(s.obj.contains("load i32, ptr @lust.rt.lust_arc_empty.h"));
        assert!(s.obj.contains("@llvm.used = appending"));
        assert!(s.obj.contains("define dso_local i64 @big("));
    }

    #[test]
    fn the_copy_has_small_bodies_and_declares_the_rest() {
        let s = split(IR, 3);
        assert!(s.inl.contains("define available_externally hidden fastcc i32 @lust.rt.probe(ptr noundef %0) unnamed_addr #0 {"), "{}", s.inl);
        assert!(s.inl.contains("define available_externally dso_local void @lust_arc_inc("));
        assert!(s.inl.contains("declare dso_local i64 @big(i64 noundef %0) local_unnamed_addr #0\n"), "{}", s.inl);
        assert!(!s.inl.contains("%4 = add"));
        assert!(s.inl.contains("@lust.rt.lust_tls_cur = external hidden thread_local unnamed_addr global ptr, align 8"), "{}", s.inl);
        assert!(s.inl.contains("@lust_count = external global { i64, [2 x i32] }, align 8"), "{}", s.inl);
        assert!(s.inl.contains("@lust_table = available_externally local_unnamed_addr constant [2 x i8] c\"xy\", align 1"), "{}", s.inl);
        assert!(s.inl.contains("@lust.rt.lust_arc_empty.h = external hidden global [2 x i64], align 16"), "{}", s.inl);
        assert!(s.inl.contains("@.str = private unnamed_addr constant"));
        assert!(!s.inl.contains("@llvm.used"));
        assert!(s.inl.contains("declare dso_local void @fallback(ptr noundef %0) local_unnamed_addr #0\n"), "{}", s.inl);
        assert!(s.inl.contains("@stderr = external local_unnamed_addr constant ptr, align 8"));
        assert!(s.inl.contains("@lust_weak_k = external constant i32, align 4") || s.inl.contains("@lust_weak_k = external global i32, align 4"), "{}", s.inl);
        assert!(s.inl.contains("attributes #0 = { nounwind }"));
    }
}
