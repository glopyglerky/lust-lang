// A reference in a static or const initializer to a place inside another
// static points into that static's storage, as in rustc: `&TABLE[i]`,
// `&S.f`, nested projections, through a static that is itself a reference
// (`static CATALOG: &[M] = &[..]`, the ts-rust diagnostic catalog shape),
// through consts that hold a static reference, and through const fns.
use std::sync::atomic::{AtomicU32, Ordering};
struct M { code: u32, pair: (u16, [u32; 3]) }
struct W { m: M, arr: [u32; 3], t: (u8, [M; 2]) }
static ARR: [M; 3] = [M { code: 1, pair: (2, [3, 4, 5]) }, M { code: 6, pair: (7, [8, 9, 10]) }, M { code: 11, pair: (12, [13, 14, 15]) }];
static CATALOG: &[M] = &[M { code: 21, pair: (22, [23, 24, 25]) }, M { code: 26, pair: (27, [28, 29, 30]) }];
static ST: W = W { m: M { code: 41, pair: (42, [43, 44, 45]) }, arr: [46, 47, 48], t: (49, [M { code: 50, pair: (51, [52, 53, 54]) }, M { code: 55, pair: (56, [57, 58, 59]) }]) };
static RS: &[M] = &ARR;
static PS: &W = &ST;
static NEST: &(&[M], u8) = &(&[M { code: 61, pair: (62, [63, 64, 65]) }], 71);
static RPT: [&[u32; 2]; 3] = [&[81, 82]; 3];
static BYTES: &[u8] = b"hello";
static A: &M = &ARR[1];
static B: &M = &ARR[1];
static KIND: &M = &CATALOG[1];
static CODE: &u32 = &KIND.code;
static DEEP: &u32 = &ST.t.1[1].pair.1[2];
static FIELD: &[u32; 3] = &ST.arr;
static SLICE: &[u32] = &ST.arr;
static VIA_REF: &M = &RS[2];
static VIA_PTR: &M = &PS.t.1[0];
static VIA_DEREF: &M = &(*PS).m;
static NESTED: &u32 = &NEST.0[0].pair.1[1];
static SHARED: &u32 = &RPT[2][1];
static BYTE: &u8 = &BYTES[1];
static IN_TUPLE: (&M, u8) = (&CATALOG[0], 9);
static IN_ARRAY: [&u32; 2] = [&ARR[0].code, &ARR[2].code];
static IN_OPTION: Option<&M> = Some(&ARR[0]);
static LATER: &u32 = &AFTER[1];
static AFTER: [u32; 2] = [90, 91];
const C_TO_STATIC: &M = &ARR[2];
const C_SLICE: &[M] = &ARR;
static S_FROM_CONST: &M = &C_SLICE[1];
static S_FROM_CONST_FIELD: &u32 = &C_TO_STATIC.pair.1[0];
const fn pick(s: &'static [M], i: usize) -> &'static M { &s[i] }
const fn code_of(m: &'static M) -> &'static u32 { &m.code }
static PICKED: &M = pick(RS, 1);
static PICKED_CODE: &u32 = code_of(PICKED);
static ATOMS: [AtomicU32; 2] = [AtomicU32::new(1), AtomicU32::new(2)];
static ATOM: &AtomicU32 = &ATOMS[1];
fn main() {
    println!("{} {} {}", std::ptr::eq(A, &ARR[1]), std::ptr::eq(A, B), std::ptr::eq(A, &ARR[0]));
    println!("{} {}", std::ptr::eq(KIND, &CATALOG[1]), std::ptr::eq(CODE, &CATALOG[1].code));
    println!("{} {} {}", std::ptr::eq(DEEP, &ST.t.1[1].pair.1[2]), std::ptr::eq(FIELD, &ST.arr), std::ptr::eq(SLICE.as_ptr(), ST.arr.as_ptr()));
    println!("{} {} {}", std::ptr::eq(VIA_REF, &ARR[2]), std::ptr::eq(VIA_PTR, &ST.t.1[0]), std::ptr::eq(VIA_DEREF, &ST.m));
    println!("{} {} {}", std::ptr::eq(NESTED, &NEST.0[0].pair.1[1]), std::ptr::eq(SHARED, &RPT[0][1]), std::ptr::eq(BYTE, &BYTES[1]));
    println!("{} {} {}", std::ptr::eq(IN_TUPLE.0, &CATALOG[0]), std::ptr::eq(IN_ARRAY[1], &ARR[2].code), std::ptr::eq(IN_OPTION.unwrap(), &ARR[0]));
    println!("{} {}", std::ptr::eq(LATER, &AFTER[1]), std::ptr::eq(C_TO_STATIC, &ARR[2]));
    println!("{} {}", std::ptr::eq(S_FROM_CONST, &ARR[1]), std::ptr::eq(S_FROM_CONST_FIELD, &ARR[2].pair.1[0]));
    println!("{} {}", std::ptr::eq(PICKED, &ARR[1]), std::ptr::eq(PICKED_CODE, &ARR[1].code));
    ATOM.fetch_add(5, Ordering::SeqCst);
    println!("{} {}", ATOMS[1].load(Ordering::SeqCst), std::ptr::eq(ATOM, &ATOMS[1]));
    {
        static LOCAL: [u32; 2] = [7, 8];
        static LOCAL_REF: &u32 = &LOCAL[1];
        println!("{} {}", std::ptr::eq(LOCAL_REF, &LOCAL[1]), *LOCAL_REF);
    }
    println!("{} {} {} {} {} {}", A.code, CODE, DEEP, SLICE.len(), NESTED, BYTE);
    println!("{} {} {} {}", SHARED, IN_ARRAY[0], LATER, PICKED_CODE);
}
