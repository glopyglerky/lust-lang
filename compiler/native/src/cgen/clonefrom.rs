// The `clonefrom_<m>` glue: `void clonefrom_T(T *d, T *s)`, the
// `Clone::clone_from` body of a type with no written fn at the call
// (R24). A written `clone_from` (gx.clone_from_methods) is called
// where one exists. The core types that specialize `clone_from`
// (Vec, String, Box, Box<[T]>, Option, Result, arrays, RefCell,
// Cow) run those bodies, reusing allocations exactly as in std (a
// Vec or String keeps its capacity). Everything else, derived
// Adts and tuples included, takes core's default: clone into a
// temp, drop the old value, assign. Derived `Clone` does not
// derive `clone_from`.
use super::*;

/// True when `t` has a written `Clone::clone` the glue calls
/// (the clone glue's `written` test).
fn written_clone(p: &Program, t: &CTy) -> bool {
    matches!(t, CTy::Adt(a) if p.gx.glue_methods.contains_key(&(*a, p.gx.lang.clone)))
}

/// True when `t` has a written `Clone::clone_from` the glue calls.
fn written_cf(p: &Program, t: &CTy) -> bool {
    matches!(t, CTy::Adt(a) if p.gx.clone_from_methods.contains_key(a))
}

/// One `clone_from` of the place `dl` from the place `sl` (both C
/// lvalue expressions of type `t`), as a C statement: a plain copy
/// for a Copy type with no written Clone impl, else the glue call.
fn cf_place(p: &Program, t: &CTy, dl: &str, sl: &str, need: &mut dyn FnMut(Glue, &CTy)) -> String {
    if is_copy_c(p, t) && !written_clone(p, t) && !written_cf(p, t) {
        format!("{dl} = {sl};")
    } else {
        need(Glue::CloneFrom, t);
        format!("clonefrom_{}(&{dl}, &{sl});", mangle(t))
    }
}

/// A clone of the place `sl` of type `t`, as a C value expression
/// (the clone glue's `clone_of` rule).
fn clone_place(p: &Program, t: &CTy, sl: &str, need: &mut dyn FnMut(Glue, &CTy)) -> String {
    if is_copy_c(p, t) && !written_clone(p, t) {
        sl.to_string()
    } else {
        need(Glue::Clone, t);
        format!("clone_{}(&{sl})", mangle(t))
    }
}

/// core's default `clone_from` body for the whole value: clone the
/// source into a temporary, drop the destination's old value, then
/// assign, in that order (a panicking clone leaves `d` untouched).
fn default_body(p: &Program, t: &CTy, ct: &str, need: &mut dyn FnMut(Glue, &CTy)) -> String {
    if is_copy_c(p, t) && !written_clone(p, t) {
        return "  *d = *s;\n".to_string();
    }
    need(Glue::Clone, t);
    let drop = if needs_drop_c(p, t) {
        need(Glue::Drop, t);
        format!("drop_{}(d); ", mangle(t))
    } else {
        String::new()
    };
    format!("  {{ {ct} t_ = clone_{}(s); {drop}*d = t_; }}\n", mangle(t))
}

/// The default as a statement over two places (an enum whose
/// variant changed, a boxed slice whose length changed).
fn default_place(p: &Program, t: &CTy, dl: &str, sl: &str, need: &mut dyn FnMut(Glue, &CTy)) -> String {
    if is_copy_c(p, t) && !written_clone(p, t) {
        return format!("{dl} = {sl};");
    }
    let c = clone_place(p, t, sl, need);
    let drop = if needs_drop_c(p, t) {
        need(Glue::Drop, t);
        format!("drop_{}(&{dl}); ", mangle(t))
    } else {
        String::new()
    };
    format!("{{ {} t_ = {c}; {drop}{dl} = t_; }}", ctype(t))
}

/// Generates the prototype and definition of the `clonefrom` glue
/// of `t` (the `Glue::CloneFrom` arm of `glue_def_value`).
#[allow(clippy::too_many_arguments)]
pub(super) fn clonefrom_glue(
    p: &Program,
    slots: &Slots,
    t: &CTy,
    protos: &mut String,
    defs: &mut String,
    need: &mut dyn FnMut(Glue, &CTy),
    types: &mut Vec<CTy>,
) {
    let m = mangle(t);
    let ct = ctype(t);
    let _ = writeln!(protos, "void clonefrom_{m}({ct} *d, {ct} *s);");
    // A written `clone_from` is the body.
    if let CTy::Adt(a) = t
        && let Some(&f) = p.gx.clone_from_methods.get(a)
    {
        let callee = glue_callee(p, slots, f, types);
        let _ = writeln!(defs, "void clonefrom_{m}({ct} *d, {ct} *s) {{\n  {callee}(d, s);\n}}\n");
        return;
    }
    let mut body = String::new();
    // A written `clone` with no `clone_from` takes core's default,
    // and so does every shape with no body below.
    let structural = !matches!(t, CTy::Adt(a) if p.gx.glue_methods.contains_key(&(*a, p.gx.lang.clone)));
    if !structural {
        body.push_str(&default_body(p, t, &ct, need));
    } else {
        match t {
            CTy::String => {
                body.push_str("  d->len = 0;\n  lust_string_push_str(d, lust_string_as_str(s));\n");
            }
            CTy::Vec(e) => {
                let et = ctype(e);
                if needs_drop_c(p, e) {
                    need(Glue::Drop, e);
                    let dm = mangle(e);
                    let _ = writeln!(
                        body,
                        "  if (d->len > s->len) {{ for (uint64_t i = s->len; i < d->len; i++) drop_{dm}(&(({et}*)d->ptr)[i]); d->len = s->len; }}"
                    );
                } else {
                    body.push_str("  if (d->len > s->len) d->len = s->len;\n");
                }
                let cf = cf_place(p, e, &format!("(({et}*)d->ptr)[i]"), &format!("(({et}*)s->ptr)[i]"), need);
                let c = clone_place(p, e, &format!("(({et}*)s->ptr)[i]"), need);
                let _ = writeln!(body, "  for (uint64_t i = 0; i < d->len; i++) {cf}");
                let _ = writeln!(
                    body,
                    "  if (s->len > d->len) {{ lust_vec_reserve(d, s->len - d->len, sizeof({et}), _Alignof({et})); for (uint64_t i = d->len; i < s->len; i++) (({et}*)d->ptr)[i] = {c}; d->len = s->len; }}"
                );
            }
            CTy::Boxed(e) => {
                let st = cf_place(p, e, "(**d)", "(**s)", need);
                let _ = writeln!(body, "  {st}");
            }
            CTy::BoxSlice(e) => {
                let cf = cf_place(p, e, "d->ptr[i]", "s->ptr[i]", need);
                let def = default_place(p, t, "(*d)", "(*s)", need);
                let _ = writeln!(body, "  if (d->len != s->len) {{ {def} }} else {{ for (uint64_t i = 0; i < d->len; i++) {cf} }}");
            }
            CTy::Option(e) => {
                let (dd, ds) = (disc_call(p, t, "d"), disc_call(p, t, "s"));
                let cf = cf_place(p, e, &variant_field(p, t, "(*d)", 1, 0), &variant_field(p, t, "(*s)", 1, 0), need);
                let def = default_place(p, t, "(*d)", "(*s)", need);
                let _ = writeln!(body, "  if ({dd} != {ds}) {{ {def} }} else if ({dd}) {{ {cf} }}");
            }
            CTy::Result(a, b) => {
                let (dd, ds) = (disc_call(p, t, "d"), disc_call(p, t, "s"));
                let ok = cf_place(p, a, &variant_field(p, t, "(*d)", 0, 0), &variant_field(p, t, "(*s)", 0, 0), need);
                let err = cf_place(p, b, &variant_field(p, t, "(*d)", 1, 0), &variant_field(p, t, "(*s)", 1, 0), need);
                let def = default_place(p, t, "(*d)", "(*s)", need);
                let _ = writeln!(body, "  if ({dd} != {ds}) {{ {def} }} else if (!{dd}) {{ {ok} }} else {{ {err} }}");
            }
            // A `RefCell` clone_froms its value under a mutable
            // borrow of the destination and a shared borrow of the
            // source, with the borrows' panic texts.
            CTy::RefCell(e) => {
                let st = cf_place(p, e, "d->f1", "s->f1", need);
                let _ = writeln!(
                    body,
                    "  if (d->f0 != 0) lust_panic_cstr_dyn(\"RefCell already borrowed\", 0);\n  if (s->f0 < 0) lust_panic_cstr_dyn(\"RefCell already mutably borrowed\", 0);\n  d->f0 = -1; {st} d->f0 = 0;"
                );
            }
            CTy::Array(e, n) => {
                let st = cf_place(p, e, "d->v[i]", "s->v[i]", need);
                let _ = writeln!(body, "  for (uint64_t i = 0; i < {n}; i++) {st}");
            }
            // Derived `Clone` does not derive `clone_from`: an Adt
            // takes core's default (assign a clone), as tuples do.
            // The one Adt here with its own body is `Cow`, whose
            // alloc `clone_from` reuses an Owned buffer in place:
            // the same-variant field body below.
            CTy::Adt(a) if fmt::cow_view(p, t, "d", "x").is_some() => {
                let adt = p.adt(*a);
                if adt.is_enum {
                    let (dd, ds) = (disc_call(p, t, "d"), disc_call(p, t, "s"));
                    let def = default_place(p, t, "(*d)", "(*s)", need);
                    let _ = writeln!(body, "  if ({dd} != {ds}) {{ {def} }} else {{");
                    let _ = writeln!(body, "  switch ({dd}) {{");
                    for (vi, v) in adt.variants.iter().enumerate() {
                        let _ = write!(body, "  case {vi}:");
                        for (fi, f) in v.fields.iter().enumerate() {
                            let ft = cty(&p.tys, f.ty);
                            let st = cf_place(p, &ft, &variant_field(p, t, "(*d)", vi, fi), &variant_field(p, t, "(*s)", vi, fi), need);
                            let _ = write!(body, " {st}");
                        }
                        body.push_str(" break;\n");
                    }
                    body.push_str("  }\n  }\n");
                } else {
                    for (fi, f) in adt.variants[0].fields.iter().enumerate() {
                        let ft = cty(&p.tys, f.ty);
                        let st = cf_place(p, &ft, &format!("d->f{fi}"), &format!("s->f{fi}"), need);
                        let _ = writeln!(body, "  {st}");
                    }
                }
            }
            _ => body.push_str(&default_body(p, t, &ct, need)),
        }
    }
    let _ = writeln!(defs, "void clonefrom_{m}({ct} *d, {ct} *s) {{\n{body}}}\n");
}
