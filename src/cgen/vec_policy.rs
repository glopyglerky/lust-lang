// RawVec's minimum nonzero capacity is a Rust-layout property. Its C
// allocation still uses the physical element size and alignment.
use super::*;

thread_local! {
    // A phase belongs to one frozen Program, including its target layout.
    // Never share an answer across bodies while recording layout reads.
    static MIN_CAPS: std::cell::RefCell<(u64, crate::intern::FxMap<CTy, Option<u64>>)> =
        std::cell::RefCell::new((0, crate::intern::FxMap::default()));
}

fn minimum(size: u64) -> u64 {
    if size == 1 { 8 } else if size <= 1024 { 4 } else { 1 }
}

/// A capacity-policy expression, separate from the C storage layout.
/// Modeled semantic layouts produce a constant. Shapes outside that
/// grammar keep the previous C-size policy; this is not a claim that
/// their C layout matches Rust (see ARCH 4.4).
pub(super) fn min_capacity(p: &Program, t: &CTy) -> std::borrow::Cow<'static, str> {
    match semantic_min(p, t) {
        Some(1) => "1".into(),
        Some(8) => "8".into(),
        Some(_) => "4".into(),
        None => {
            let ct = ctype(t);
            format!("(sizeof({ct}) == 1 ? 8 : sizeof({ct}) <= 1024 ? 4 : 1)").into()
        }
    }
}

fn semantic_min(p: &Program, t: &CTy) -> Option<u64> {
    // The common leaf types need neither an overlay nor a layout walk.
    match t {
        CTy::Unit => return Some(4),
        CTy::Bool => return Some(8),
        CTy::Int(i) => return Some(minimum(layout::int_layout(*i).size)),
        CTy::Char | CTy::Float(_) | CTy::FnPtr(..) | CTy::Ref(_) | CTy::Ptr(_)
        | CTy::FatPtr(_) | CTy::Str | CTy::Slice(_) | CTy::String | CTy::Vec(_)
        | CTy::Boxed(_) | CTy::BoxSlice(_) | CTy::Rc(_) | CTy::Weak(_)
        | CTy::Dyn | CTy::BoxDyn | CTy::RcDyn | CTy::WeakDyn(_) | CTy::WeakOwn(..)
        | CTy::Own(..) => return Some(4),
        CTy::Sync(crate::ty::sync::SyncTy::Arc | crate::ty::sync::SyncTy::ArcWeak, _) => return Some(4),
        _ => {}
    }
    let tag = if crate::check::derive_delta::open() { 0 } else { phase_tag() };
    if tag != 0
        && let Some(hit) = MIN_CAPS.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.0 != tag {
                *cache = (tag, crate::intern::FxMap::default());
            }
            cache.1.get(t).copied()
        })
    {
        return hit;
    }
    let mut tys = Tys::new(&p.tys);
    let result = semantic_ty(&mut tys, t)
        .and_then(|t| layout::layout_of_ty(&tys, &Shapes(p), t).ok())
        .map(|l| minimum(l.size));
    if tag != 0 {
        MIN_CAPS.with(|cache| cache.borrow_mut().1.insert(t.clone(), result));
    }
    result
}

/// Numeric policy literals depend on these shapes in dev C as well as
/// release C. The exact-layout adapter boundary is the same one used by
/// user-visible size queries; an approximate source shim is not promoted.
struct Shapes<'a>(&'a Program);

impl layout::AdtShapes for Shapes<'_> {
    fn adt_shape(&self, id: AdtId) -> Option<layout::AdtShape> {
        self.0.note_adt_layout(id);
        layout::AdtShapes::adt_shape(&UserLayout(self.0), id)
    }
}

/// Reconstitutes only the layout-bearing structure erased by `cty`.
/// References and owning pointers use a representative pointee of the same thin or
/// fat shape; their pointee's layout cannot affect this value's layout.
/// Tuples can also be erased iterators/closures, and u64 can be an erased
/// LocalKey with a pointer niche: neither has enough original provenance
/// to reconstruct inside an aggregate. These and unsupported iterator,
/// Fmt and guard forms return None. An original user ADT's field types
/// retain their provenance and go directly through the shared engine.
fn semantic_ty(tys: &mut Tys<'_>, t: &CTy) -> Option<Ty> {
    let ptr = |tys: &mut Tys<'_>| tys.mk_ref(false, UNIT);
    let fat = |tys: &mut Tys<'_>| {
        let slice = tys.intern(TyKind::Slice(int(IntTy::U8)));
        tys.mk_ref(false, slice)
    };
    Some(match t {
        CTy::Unit => UNIT,
        CTy::Bool => BOOL,
        CTy::Char => CHAR,
        CTy::Int(IntTy::U64) => return None,
        CTy::Int(i) => tys.intern(TyKind::Int(*i)),
        CTy::Float(f) => tys.intern(TyKind::Float(*f)),
        CTy::FnPtr(..) => ptr(tys),
        CTy::Ref(e) if **e == CTy::Str => fat(tys),
        CTy::Ref(e) if sized_pointee(e) => ptr(tys),
        CTy::Boxed(e) | CTy::Rc(e) | CTy::Weak(e) if sized_pointee(e) => ptr(tys),
        CTy::Ptr(e) if sized_pointee(e) => tys.intern(TyKind::RawPtr(Mut::Shared, UNIT)),
        CTy::FatPtr(e) if **e == CTy::Dyn => {
            let bounds = tys.intern_dyn_bounds(DynBounds { principal: None, supertraits: Box::new([]), auto: AutoTraits::default() });
            let dyn_ty = tys.intern(TyKind::Dyn(bounds, Region::Erased));
            tys.intern(TyKind::RawPtr(Mut::Shared, dyn_ty))
        }
        CTy::FatPtr(_) => {
            let slice = tys.intern(TyKind::Slice(int(IntTy::U8)));
            tys.intern(TyKind::RawPtr(Mut::Shared, slice))
        }
        CTy::Str | CTy::Slice(_) | CTy::BoxSlice(_) | CTy::Dyn | CTy::BoxDyn
        | CTy::RcDyn | CTy::WeakDyn(_) | CTy::WeakOwn(..) | CTy::Own(..) => fat(tys),
        CTy::String | CTy::Vec(_) => tys.mk_vec(UNIT),
        CTy::Array(e, n) => {
            let e = semantic_ty(tys, e)?;
            let n = tys.intern_const(ConstKind::Int(*n as u128));
            tys.intern(TyKind::Array(e, n))
        }
        CTy::Option(e) => {
            let e = semantic_ty(tys, e)?;
            tys.mk_option(e)
        }
        CTy::Result(a, b) => {
            let (a, b) = (semantic_ty(tys, a)?, semantic_ty(tys, b)?);
            tys.mk_adt(tys.builtins().result, &[GenericArg::Ty(a), GenericArg::Ty(b)])
        }
        CTy::Adt(a) => tys.adt_ty(*a),
        CTy::Cell(e) | CTy::RefCell(e) => {
            let e = semantic_ty(tys, e)?;
            let def = if matches!(t, CTy::Cell(_)) { tys.builtins().cell } else { tys.builtins().refcell };
            tys.mk_adt(def, &[GenericArg::Ty(e)])
        }
        CTy::Sync(k, e) => {
            use crate::ty::sync::SyncTy;
            let e = match k {
                SyncTy::Mutex | SyncTy::RwLock | SyncTy::OnceLock => semantic_ty(tys, e)?,
                SyncTy::Arc | SyncTy::ArcWeak if !sized_pointee(e) => return None,
                // The modeled remaining sync types have no inline T.
                _ => UNIT,
            };
            tys.mk_adt(tys.builtins().sync_tys[*k as usize], &[GenericArg::Ty(e)])
        }
        _ => return None,
    })
}

/// `cty` does not yet encode tuple-tail metadata. A direct pointer's
/// capacity class is 4 for either width, but an enclosing array/aggregate
/// needs the exact width. Do not reconstruct an ambiguous pointer as thin.
fn sized_pointee(t: &CTy) -> bool {
    match t {
        CTy::Tuple(fields) => fields.iter().all(sized_pointee),
        CTy::Array(e, _) | CTy::Cell(e) | CTy::RefCell(e) => sized_pointee(e),
        CTy::Sync(crate::ty::sync::SyncTy::Mutex | crate::ty::sync::SyncTy::RwLock, e) => sized_pointee(e),
        CTy::Unit | CTy::Bool | CTy::Char | CTy::Int(_) | CTy::Float(_) | CTy::FnPtr(..)
        | CTy::Ref(_) | CTy::Ptr(_) | CTy::FatPtr(_) | CTy::String | CTy::Vec(_)
        | CTy::Option(_) | CTy::Result(..) | CTy::Boxed(_) | CTy::BoxSlice(_)
        | CTy::Rc(_) | CTy::Weak(_) | CTy::BoxDyn | CTy::RcDyn | CTy::WeakDyn(_)
        | CTy::WeakOwn(..) | CTy::Own(..) => true,
        // Str/Slice/Dyn views, unknown ADTs and the remaining compact
        // backend forms have no Sized proof in this representation.
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::derive_delta;
    use crate::diag::FileId;
    use crate::intern::{Interner, Known};

    fn program(src: &str) -> Program {
        let interner = std::sync::Arc::new(Interner::default());
        let known = std::sync::Arc::new(Known::new(&interner));
        let lexed = crate::lexer::lex(src, FileId(0), 0, &interner).unwrap();
        let ast = crate::parser::parse(&lexed, &interner, &known).unwrap();
        let ids = crate::def::CrateIds { local: crate::def::LOCAL_CRATE, stable: crate::def::synthetic_crate_hash("policy.rs", false) };
        crate::check::collect::collect(std::sync::Arc::new(ast), interner, known, ids).unwrap()
    }

    #[test]
    fn policy_uses_the_program_target_without_changing_c_storage() {
        use crate::ty::layout::SyncLayout;
        use crate::ty::sync::SyncTy;
        let mut p = program("fn main() {}\n");
        let bytes = |n| CTy::Array(Box::new(CTy::Int(IntTy::U8)), n);
        let lock = CTy::Sync(SyncTy::Mutex, Box::new(bytes(1016)));
        let once = CTy::Sync(SyncTy::OnceLock, Box::new(bytes(1020)));
        for target in [SyncLayout::Futex, SyncLayout::Darwin, SyncLayout::Windows, SyncLayout::Futex] {
            p.tys.sync_layout = target;
            in_layout_phase(layout_phase(), || {
                let expected = if target == SyncLayout::Darwin { 1 } else { 4 };
                for t in [&lock, &once] {
                    assert_eq!(semantic_min(&p, t), Some(expected));
                    assert_eq!(semantic_min(&p, t), Some(expected));
                    assert_eq!(c_size_align(&p, t), (1024, 4));
                }
                let nested = CTy::Array(Box::new(CTy::Sync(SyncTy::Mutex, Box::new(CTy::Unit))), 65);
                assert_eq!(semantic_min(&p, &nested), Some(expected));
                assert_eq!(c_size_align(&p, &nested), (520, 4));
            });
        }
    }

    #[test]
    fn cached_policy_does_not_skip_dev_layout_reads() {
        let p = program("struct Inner { bytes: [u8; 1025] } struct Outer { value: Inner } fn main() {}\n");
        let inner = p.type_names[&p.interner.intern("Inner")];
        let outer = p.type_names[&p.interner.intern("Outer")];
        let t = CTy::Adt(outer);
        names::with(None, || in_layout_phase(layout_phase(), || {
            assert_eq!(semantic_min(&p, &t), Some(1));
            for _ in 0..2 {
                let (answer, reads) = derive_delta::scoped(true, || semantic_min(&p, &t));
                assert_eq!(answer, Some(1));
                for a in [inner, outer] {
                    assert!(reads.iter().any(|&(d, bits)| d == p.def_of_adt(a) && bits & derive_delta::bit::LAYOUT != 0));
                }
            }
        }));
    }

    #[test]
    fn nested_ambiguous_pointers_keep_the_legacy_boundary() {
        use crate::ty::sync::SyncTy;
        let p = program("fn main() {}\n");
        let tail = Box::new(CTy::Tuple(vec![CTy::Int(IntTy::U8), CTy::Str]));
        for pointer in [
            CTy::Ref(tail.clone()), CTy::Ptr(tail.clone()), CTy::Boxed(tail.clone()),
            CTy::Rc(tail.clone()), CTy::Weak(tail.clone()), CTy::Sync(SyncTy::Arc, tail.clone()),
            CTy::Sync(SyncTy::ArcWeak, tail),
        ] {
            assert_eq!(semantic_min(&p, &pointer), Some(4));
            let array = CTy::Array(Box::new(pointer), 65);
            assert_eq!(semantic_min(&p, &array), None);
            assert!(min_capacity(&p, &array).contains("sizeof("));
        }
        let sized = CTy::Ptr(Box::new(CTy::Tuple(vec![CTy::Int(IntTy::U8), CTy::Int(IntTy::U16)])));
        assert_eq!(semantic_min(&p, &CTy::Array(Box::new(sized), 65)), Some(4));
        assert_eq!(semantic_min(&p, &CTy::VecDrain(Box::new(CTy::Int(IntTy::U8)))), None);
    }

    #[test]
    fn erased_origins_are_not_promoted_to_semantic_layouts() {
        let p = program("fn main() {}\n");
        let tuple = CTy::Tuple(vec![CTy::Int(IntTy::U8)]);
        assert_eq!(semantic_min(&p, &tuple), None);
        assert_eq!(semantic_min(&p, &CTy::Array(Box::new(tuple), 1025)), None);
        let local_key_or_u64 = CTy::Option(Box::new(CTy::Int(IntTy::U64)));
        assert_eq!(semantic_min(&p, &CTy::Array(Box::new(local_key_or_u64), 65)), None);
        assert_eq!(semantic_min(&p, &CTy::Int(IntTy::U64)), Some(4));
    }
}
