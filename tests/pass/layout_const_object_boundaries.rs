// Layout queries do not construct values of these large types.
#![allow(dead_code)]
type Last = [u8; core::mem::size_of::<[u8; 2305843009213693951]>()];
fn force_alias_evaluation(_: Last) {}

// The older alias evaluator retains its builtin type coverage.
type OptionBytes = [u8; core::mem::size_of::<Option<u32>>()];

fn signature() -> [u8; core::mem::align_of::<[u8; 2305843009213693951]>()] { [0; 1] }

fn main() {
    assert_eq!(signature().len(), 1);
    assert_eq!(core::mem::size_of::<[u8; 2305843009213693951]>(), 2305843009213693951);
    assert_eq!(core::mem::size_of::<([u8; 2305843009213693936], u64)>(), 2305843009213693944);
    assert_eq!(core::mem::size_of::<([u8; 2305843009213693944], [u64; 0])>(), 2305843009213693944);
    assert_eq!(core::mem::size_of::<[(); 18446744073709551615]>(), 0);
    let bytes: OptionBytes = [0; 8];
    assert_eq!(bytes.len(), 8);
}
